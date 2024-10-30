#![no_std]
extern crate alloc;
mod address;
mod rv39;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use core::fmt::{self, Debug, Formatter};
use rv39::{PAGE_SIZE, PAGE_TABLE_LEVEL, PTE_ENTRY_NUM};

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl From<PhysPageNum> for FrameTracker {
    fn from(v: PhysPageNum) -> Self {
        FrameTracker { ppn: v }
    }
}

impl From<FrameTracker> for PhysPageNum {
    fn from(v: FrameTracker) -> Self {
        v.ppn
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn get_pte_array(&self) -> Option<&mut [PageTableEntry]> {
        if self.is_valid() && !self.readable() && !self.writable() && !self.executable() {
            return Some(self.ppn().get_pte_array());
        }
        None
    }
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.bits >> 10 & ((1usize << 44) - 1))
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

pub trait AllocPageFrame {
    fn page_frame_alloc(&self) -> Option<FrameTracker>;
    fn page_frame_dealloc(&self, frame: FrameTracker);
}

pub struct PageTable<A: AllocPageFrame> {
    allocator: A,
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl<A: AllocPageFrame> Debug for PageTable<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const FORMAT_STR: [&'static str; 4] = ["   ", "  ", " ", ""];
        fn walk(
            pte: &PageTableEntry,
            va: VirtPageNum,
            level: u32,
            f: &mut Formatter<'_>,
        ) -> fmt::Result {
            if pte.is_valid() {
                let start_va: VirtAddr = va.into();
                let end_va: VirtAddr = va.nth(PTE_ENTRY_NUM.pow(level)).into();
                if let Some(pte_array) = pte.get_pte_array() {
                    writeln!(
                        f,
                        "{}{:?}-{:?}",
                        FORMAT_STR[level as usize], start_va, end_va
                    )?;
                    let mut cur_va = va;
                    for cur_pte in pte_array {
                        walk(cur_pte, cur_va, level - 1, f)?;
                        cur_va = cur_va.nth(PTE_ENTRY_NUM.pow(level - 1));
                    }
                } else {
                    writeln!(
                        f,
                        "{}{:?}<->{:?} {:?}",
                        FORMAT_STR[level as usize],
                        VirtAddr::from(va),
                        PhysAddr::from(pte.ppn()),
                        pte.flags()
                    )?;
                }
            }
            Ok(())
        }
        let root = PageTableEntry::new(self.root_ppn, PTEFlags::V);
        walk(&root, VirtPageNum(0), PAGE_TABLE_LEVEL, f)?;
        Ok(())
    }
}

pub trait PageMap {
    fn map_one(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags);
    fn unmap_one(&mut self, vpn: VirtPageNum);
    /// size:bytes
    fn map(&mut self, mut vpn: VirtPageNum, mut ppn: PhysPageNum, size: usize, flags: PTEFlags) {
        let count = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        for _ in 0..count {
            self.map_one(vpn, ppn, flags);
            vpn = vpn.nth(1);
            ppn = ppn.nth(1);
        }
    }

    fn unmap(&mut self, mut vpn: VirtPageNum, size: usize) {
        let count = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        for _ in 0..count {
            self.unmap_one(vpn);
            vpn = vpn.nth(1);
        }
    }
}

/// Assume that it won't oom when creating/mapping.
impl<T: AllocPageFrame> PageTable<T> {
    pub fn new(allocator: T) -> Self {
        let frame = allocator.page_frame_alloc().unwrap();
        PageTable {
            allocator,
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize, allocator: T) -> Self {
        Self {
            allocator,
            root_ppn: PhysPageNum(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    fn find_pte(&mut self, vpn: VirtPageNum, alloc: bool) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                if alloc {
                    let frame = self.allocator.page_frame_alloc().unwrap();
                    *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                    self.frames.push(frame);
                } else {
                    return None;
                }
            }
            ppn = pte.ppn();
        }
        result
    }

    // pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    //     self.find_pte(vpn).map(|pte| *pte)
    // }
    // pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
    //     self.find_pte(va.clone().floor()).map(|pte| {
    //         let aligned_pa: PhysAddr = pte.ppn().into();
    //         let offset = va.page_offset();
    //         let aligned_pa_usize: usize = aligned_pa.into();
    //         (aligned_pa_usize + offset).into()
    //     })
    // }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

impl<T: AllocPageFrame> PageMap for PageTable<T> {
    fn map_one(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte(vpn, true).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    fn unmap_one(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn, false).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
}
