#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(const_trait_impl)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod hal;
mod kvm;
mod lang;
mod logging;
mod mm;
mod sync;
mod syscall;

use core::arch::global_asm;

use sbi_rt::{system_reset, NoReason, Shutdown};

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn rust_main() -> ! {
    clear_bss();
    mm::init();
    logging::init();
    kvm::init();
    println!("[kernel] Hello, world!");
    system_reset(Shutdown, NoReason);
    unreachable!()
}
