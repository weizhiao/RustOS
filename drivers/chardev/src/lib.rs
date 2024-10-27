#![no_std]
mod ns16550a;
pub use ns16550a::NS16550a;

pub trait CharDevice {
    fn init(&self);
    fn read(&self) -> Option<u8>;
    fn write(&self, ch: u8);
    fn handle_irq(&self);
}
