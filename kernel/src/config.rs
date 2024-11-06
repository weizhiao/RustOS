use pagetable::PAGE_SIZE;

pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;
pub const MEMORY_END: usize = 0x8800_0000;
pub const PER_STACK_SIZE: usize = 4096 * 8;
pub const NCPU: usize = 16;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
