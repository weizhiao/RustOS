use core::panic::PanicInfo;

use sbi_rt::{system_reset, NoReason, Shutdown};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message());
    }
    system_reset(Shutdown, NoReason);
    unreachable!()
}
