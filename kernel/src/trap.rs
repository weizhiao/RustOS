use trap::{kernel_trap_vec, Interrupt, TrapControl, TrapHandler, TrapKind};
use trap_macro::kernel_trap;

type TrapContext = trap::TrapContext<()>;

pub fn init() {
    TrapHandler::set_trap_entry(kernel_trap_vec as usize);
}

kernel_trap_vec!(intr_handle);

#[no_mangle]
#[kernel_trap(context = TrapContext)]
extern "C" fn intr_handle(_trap_cx: &TrapContext) {
    let cause = TrapHandler::cause();
    match cause.kind {
        TrapKind::Interrupt(Interrupt::SupervisorExternal) => {
            board::irq_handler();
        }
        _ => {
            panic!("Unsupported trap from kernel: {:?}", cause,);
        }
    }
}
