mod frame_allocator;
mod heap_allocator;
pub use frame_allocator::{FrameAllocator, PageAllocator, FRAME_ALLOCATOR};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
}
