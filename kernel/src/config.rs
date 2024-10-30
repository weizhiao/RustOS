pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;
pub const MEMORY_END: usize = 0x8800_0000;
pub const BOOT_STACK_SIZE: usize = 4096 * 8;
pub const HART_STACK_SIZE: usize = 4096 * 4;
pub const NCPU: usize = 16;
