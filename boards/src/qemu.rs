use chardev::NS16550a;
use plic::{TargetPriority, PLIC};
use spin::Mutex;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),     // core local interrupter (CLINT)
    (0xc000000, 0x210000),    // VIRT_PLIC in virt machine
    (0x10000000, 0x9000),     // VIRT_UART0 with GPU  in virt machine
];

const VIRT_PLIC: usize = 0xC00_0000;
const VIRT_UART: usize = 0x1000_0000;

pub fn device_init() {
    use riscv::register::sie;
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let hart_id: usize = 0;
    let supervisor = TargetPriority::Supervisor;
    let machine = TargetPriority::Machine;
    plic.set_context_threshold(hart_id, supervisor, 0);
    plic.set_context_threshold(hart_id, machine, 1);
    //irq nums: 5 keyboard, 6 mouse, 8 block, 10 uart
    for intr_id in [5usize, 6, 8, 10] {
        plic.context_intr_enable(hart_id, supervisor, intr_id);
        plic.set_intr_priority(intr_id, 1);
    }
    unsafe {
        sie::set_sext();
    }
}

pub static UART: Mutex<NS16550a> = Mutex::new(NS16550a::new(VIRT_UART));

pub fn irq_handler() {
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let intr_id = plic.claim(0, TargetPriority::Supervisor);
    match intr_id {
        //10 => UART.handle_irq(),
        _ => panic!("unsupported IRQ {}", intr_id),
    }
    //plic.complete(0, TargetPriority::Supervisor, intr_id);
}
