#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(const_trait_impl)]
#![feature(fn_align)]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod kvm;
mod lang;
mod logging;
mod mm;
mod sync;
mod syscall;
mod trap;

use config::{NCPU, PER_STACK_SIZE};
use core::arch::naked_asm;
use dtb_walker::{Dtb, DtbObj, HeaderError::*, Property, Str, WalkOperation::*};

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
static BOOT_STACK: [u8; PER_STACK_SIZE * NCPU] = [0; PER_STACK_SIZE * NCPU];

#[link_section = ".text.entry"]
#[no_mangle]
#[naked]
unsafe extern "C" fn primary_entry() {
    naked_asm!(
        "
        la sp, {stack}
        addi t0, a0, 1
        li t1, {size}
        mul t1, t0, t1
        add sp, sp, t1
        call {boot}
        ",
        stack = sym BOOT_STACK,
        size = const PER_STACK_SIZE,
        boot = sym boot_primary_hart,
    );
}

#[naked]
unsafe extern "C" fn secondary_entry(hartid: usize) -> ! {
    naked_asm!(
        "
        la sp, {stack}
        addi t0, a0, 1
        li t1, {size}
        mul t1, t0, t1
        add sp, sp, t1
        call {boot}
        ",
        stack = sym BOOT_STACK,
        size = const PER_STACK_SIZE,
        boot = sym boot_secondary_harts,
    )
}

#[no_mangle]
fn boot_primary_hart(hartid: usize, device_tree_addr: usize) -> ! {
    clear_bss();
    mm::init();
    logging::init();
    kvm::init();
    board::device_init();
    trap::init();
    println!("[kernel] Hart {} start", hartid);
    // 检查设备树
    let dtb = unsafe {
        Dtb::from_raw_parts_filtered((device_tree_addr) as _, |e| {
            matches!(e, Misaligned(4) | LastCompVersion(_))
        })
    }
    .unwrap();
    secondary_harts_start(hartid, &dtb, secondary_entry as usize);
    loop {}
}

#[no_mangle]
fn boot_secondary_harts(hartid: usize) -> ! {
    kvm::init();
    trap::init();
    println!("[kernel] Hart {} start", hartid);
    loop {}
}

// 启动副核
fn secondary_harts_start(boot_hartid: usize, dtb: &Dtb, start_addr: usize) {
    if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
        println!("HSM SBI extension is not supported for current SEE.");
        return;
    }

    let mut cpus = false;
    let mut cpu: Option<usize> = None;
    dtb.walk(|path, obj| match obj {
        DtbObj::SubNode { name } => {
            if path.is_root() {
                if name == Str::from("cpus") {
                    // 进入 cpus 节点
                    cpus = true;
                    StepInto
                } else if cpus {
                    // 已离开 cpus 节点
                    if let Some(hartid) = cpu.take() {
                        hart_start(boot_hartid, hartid, start_addr);
                    }
                    Terminate
                } else {
                    // 其他节点
                    StepOver
                }
            } else if path.name() == Str::from("cpus") {
                // 如果没有 cpu 序号，肯定是单核的
                if name == Str::from("cpu") {
                    return Terminate;
                }
                if name.starts_with("cpu@") {
                    let id: usize = usize::from_str_radix(
                        unsafe { core::str::from_utf8_unchecked(&name.as_bytes()[4..]) },
                        16,
                    )
                    .unwrap();
                    if let Some(hartid) = cpu.replace(id) {
                        hart_start(boot_hartid, hartid, start_addr);
                    }
                    StepInto
                } else {
                    StepOver
                }
            } else {
                StepOver
            }
        }
        // 状态不是 "okay" 的 cpu 不能启动
        DtbObj::Property(Property::Status(status))
            if path.name().starts_with("cpu@") && status != Str::from("okay") =>
        {
            if let Some(id) = cpu.take() {
                println!("hart{} has status: {}", id, status);
            }
            StepOut
        }
        DtbObj::Property(_) => StepOver,
    });
}

fn hart_start(boot_hartid: usize, hartid: usize, start_addr: usize) {
    if hartid != boot_hartid {
        let ret = sbi_rt::hart_start(hartid, start_addr, 0);
        if ret.is_err() {
            panic!("start hart{hartid} failed. error: {ret:?}");
        }
    }
}
