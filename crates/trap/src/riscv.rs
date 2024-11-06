use super::TrapControl;
use crate::{Exception, Interrupt, TrapKind};
use riscv::register::{
    scause, sstatus, stval,
    stvec::{self, TrapMode},
};

#[macro_export]
macro_rules! kernel_trap_vec {
    ($trap_entry:ident) => {
        pub unsafe extern "C" fn kernel_trap_vec() {
            core::arch::asm!(
                ".align 2",
                ".option push",
                ".option norvc",
                "j {default}", // exception
                "j {default}", // supervisor software
                "j {default}", // reserved
                "j {default}",  // machine    software
                "j {default}", // reserved
                "j {default}", // supervisor timer
                "j {default}", // reserved
                "j {default}",  // machine    timer
                "j {default}", // reserved
                "j {default}", // supervisor external
                "j {default}", // reserved
                "j {default}", // machine    external
                ".option pop",
                default = sym $trap_entry,
                options(noreturn)
            )
        }
    };
}

pub struct VectoredTrapHandler;

impl TrapControl for VectoredTrapHandler {
    fn set_trap_entry(entry: usize) {
        unsafe {
            stvec::write(entry, TrapMode::Vectored);
        }
    }

    fn enable_supervisor_interrupt() {
        unsafe {
            sstatus::set_sie();
        }
    }

    fn disable_supervisor_interrupt() {
        unsafe {
            sstatus::clear_sie();
        }
    }

    fn cause() -> crate::TrapCause {
        let kind = match scause::read().cause() {
            scause::Trap::Interrupt(val) => TrapKind::Interrupt(val.into()),
            scause::Trap::Exception(val) => TrapKind::Exception(val.into()),
        };
        let info = stval::read();
        crate::TrapCause { kind, info }
    }
}

impl From<usize> for Interrupt {
    fn from(value: usize) -> Self {
        match value {
            1 => Self::SupervisorSoft,
            3 => Self::MachineSoft,
            5 => Self::SupervisorTimer,
            7 => Self::MachineTimer,
            9 => Self::SupervisorExternal,
            11 => Self::MachineExternal,
            _ => Self::Unknown,
        }
    }
}

impl From<usize> for Exception {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::InstructionMisaligned,
            1 => Self::InstructionFault,
            2 => Self::IllegalInstruction,
            3 => Self::Breakpoint,
            4 => Self::LoadMisaligned,
            5 => Self::LoadFault,
            6 => Self::StoreMisaligned,
            7 => Self::StoreFault,
            8 => Self::UserEnvCall,
            9 => Self::SupervisorEnvCall,
            11 => Self::MachineEnvCall,
            12 => Self::InstructionPageFault,
            13 => Self::LoadPageFault,
            15 => Self::StorePageFault,
            _ => Self::Unknown,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct TrapContext<T> {
    pub ra: usize,      // 0..
    pub t: [usize; 7],  // 1..
    pub a: [usize; 8],  // 8..
    pub s: [usize; 12], // 16..
    pub gp: usize,      // 28..
    pub tp: usize,      // 29..
    pub sp: usize,      // 30..
    pub pc: usize,      // 31..
    pub ext: T,
}
