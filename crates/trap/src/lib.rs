#![no_std]
mod riscv;
pub use riscv::TrapContext;
pub use riscv::VectoredTrapHandler as TrapHandler;

/// Interrupt
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Interrupt {
    SupervisorSoft,
    MachineSoft,
    SupervisorTimer,
    MachineTimer,
    SupervisorExternal,
    MachineExternal,
    Unknown,
}

/// Exception
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Exception {
    InstructionMisaligned,
    InstructionFault,
    IllegalInstruction,
    Breakpoint,
    LoadMisaligned,
    LoadFault,
    StoreMisaligned,
    StoreFault,
    UserEnvCall,
    SupervisorEnvCall,
    MachineEnvCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    Unknown,
}

#[derive(Debug)]
pub enum TrapKind {
    Exception(Exception),
    Interrupt(Interrupt),
}

#[derive(Debug)]
pub struct TrapCause {
    pub kind: TrapKind,
    pub info: usize,
}

pub trait TrapControl {
    fn cause() -> TrapCause;
    fn set_trap_entry(entry: usize);
    fn enable_supervisor_interrupt();
    fn disable_supervisor_interrupt();
}
