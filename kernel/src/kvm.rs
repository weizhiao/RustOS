use core::arch::asm;

use crate::{config::MEMORY_END, mm::PageAllocator};
use pagetable::{PTEFlags, PageMap, PageTable, PhysAddr, VirtAddr};
use riscv::register::satp;
use spin::lazy::Lazy;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
}

static KERNEL_PAGETABLE: Lazy<PageTable<PageAllocator>> = Lazy::new(|| kvminit());

fn kvminit() -> PageTable<PageAllocator> {
    let mut page_table = PageTable::new(PageAllocator);
    let stext_va = VirtAddr::from(stext as usize);
    let stext_pa = PhysAddr::from(stext_va.0);
    page_table.map(
        stext_va.into(),
        stext_pa.into(),
        etext as usize - stext_va.0,
        PTEFlags::X | PTEFlags::R,
    );

    let srodata_va = VirtAddr::from(srodata as usize);
    let srodata_pa = PhysAddr::from(srodata_va.0);
    page_table.map(
        srodata_va.into(),
        srodata_pa.into(),
        erodata as usize - srodata_va.0,
        PTEFlags::R,
    );

    let sdata_va = VirtAddr::from(sdata as usize);
    let sdata_pa = PhysAddr::from(sdata_va.0);
    page_table.map(
        sdata_va.into(),
        sdata_pa.into(),
        edata as usize - sdata_va.0,
        PTEFlags::R | PTEFlags::W,
    );
    let sbss_va = VirtAddr::from(sbss_with_stack as usize);
    let sbss_pa = PhysAddr::from(sbss_va.0);
    page_table.map(
        sbss_va.into(),
        sbss_pa.into(),
        ebss as usize - sbss_va.0,
        PTEFlags::R | PTEFlags::W,
    );

    let ekernel_va = VirtAddr::from(ekernel as usize);
    let ekernel_pa = PhysAddr::from(ekernel_va.0);
    page_table.map(
        ekernel_va.into(),
        ekernel_pa.into(),
        MEMORY_END - ekernel_va.0,
        PTEFlags::W | PTEFlags::R,
    );
    log::info!("kernel pagetable init");
    //println!("{:?}", page_table);
    page_table
}

pub fn init() {
    let satp = KERNEL_PAGETABLE.token();
    unsafe {
        satp::write(satp);
        asm!("sfence.vma");
    }
}
