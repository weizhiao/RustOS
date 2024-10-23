#![no_std]
#![no_main]

#[macro_use]
mod console;
mod lang;
mod logging;
mod syscall;

use core::arch::global_asm;

use sbi_rt::{system_reset, NoReason, Shutdown};

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

global_asm!(include_str!("entry.asm"));

#[no_mangle]
fn rust_main() -> ! {
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    system_reset(Shutdown, NoReason);
    unreachable!()
}
