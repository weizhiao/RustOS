use crate::{config::MEMORY_END, sync::UPIntrFreeCell};
use alloc::vec::Vec;
use pagetable::{AllocPageFrame, FrameTracker, PhysAddr, PhysPageNum};

trait FrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPageNum>>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    const fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
        // println!("last {} Physical Frames.", self.end - self.current);
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPageNum>> {
        if self.current + pages >= self.end {
            None
        } else {
            self.current += pages;
            let arr: Vec<usize> = (1..pages + 1).collect();
            let v = arr.iter().map(|x| (self.current - x).into()).collect();
            Some(v)
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

pub static FRAME_ALLOCATOR: UPIntrFreeCell<FrameAllocatorImpl> =
    unsafe { UPIntrFreeCell::new(FrameAllocatorImpl::new()) };

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub struct PageAllocator;

impl AllocPageFrame for PageAllocator {
    fn page_frame_alloc(&self) -> Option<FrameTracker> {
        FRAME_ALLOCATOR
            .exclusive_access()
            .alloc()
            .map(|ppn| ppn.into())
    }

    fn page_frame_dealloc(&self, frame: FrameTracker) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(frame.into());
    }
}

// #[allow(unused)]
// pub fn frame_allocator_test() {
//     let mut v: Vec<FrameTracker> = Vec::new();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     v.clear();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     drop(v);
//     println!("frame_allocator_test passed!");
// }

// #[allow(unused)]
// pub fn frame_allocator_alloc_more_test() {
//     let mut v: Vec<FrameTracker> = Vec::new();
//     let frames = frame_alloc_more(5).unwrap();
//     for frame in &frames {
//         println!("{:?}", frame);
//     }
//     v.extend(frames);
//     v.clear();
//     let frames = frame_alloc_more(5).unwrap();
//     for frame in &frames {
//         println!("{:?}", frame);
//     }
//     drop(v);
//     println!("frame_allocator_test passed!");
// }
