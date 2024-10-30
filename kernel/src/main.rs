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

use config::{BOOT_STACK_SIZE, HART_STACK_SIZE, NCPU};
use core::arch::asm;
use mm::{FrameAllocator, FRAME_ALLOCATOR};
use sbi_rt::{system_reset, NoReason, SbiRet, Shutdown};

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

#[allow(unused)]
#[no_mangle]
#[link_section = ".bss.stack"]
static BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0; BOOT_STACK_SIZE];

#[link_section = ".text.entry"]
#[no_mangle]
pub unsafe extern "C" fn _start() {
    asm!(
        "
        la sp, {stack}
        li a0, {size}
        add sp, sp, a0
        li a0 , 0
        call rust_main
        ",
        stack = sym BOOT_STACK,
        size = const BOOT_STACK_SIZE,
        options(nostack, nomem, noreturn),
    );
}

unsafe extern "C" fn hart_start() {
    asm!(
        "
        mv sp, a1
        call rust_main
        ",
        options(nostack, nomem, noreturn),
    );
}

fn alloc_hart_stack(hartid: usize) -> usize {
    // hartid 0使用BOOT_STACK作为栈
    #[no_mangle]
    #[link_section = ".bss.stack"]
    static HART_STACKS: [u8; HART_STACK_SIZE * (NCPU - 1)] = [0; HART_STACK_SIZE * (NCPU - 1)];

    assert!(hartid < NCPU);
    let stack_ptr = unsafe { HART_STACKS.as_ptr().add(hartid * HART_STACK_SIZE) };
    stack_ptr as usize
}

#[no_mangle]
fn rust_main(hartid: usize) -> ! {
    if hartid == 0 {
        clear_bss();
        mm::init();
        logging::init();
        kvm::init();
        println!("[kernel] Hart {} start", hartid);
        let mut hartid = 1;
        loop {
            let ret = sbi_rt::hart_start(hartid, hart_start as usize, alloc_hart_stack(hartid));
            if ret.is_err() {
                break;
            }
            hartid += 1;
        }
    } else {
        kvm::init();
        println!("[kernel] Hart {} start", hartid);
    }
    loop {}
}
