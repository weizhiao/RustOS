#![no_std]
mod qemu;

pub use chardev::CharDevice;
pub use qemu::{device_init, irq_handler, MMIO, UART};
