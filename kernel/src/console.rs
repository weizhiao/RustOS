use board::{CharDevice, UART};
use core::fmt::{self, Write};
use spin::Mutex;

struct Stdout;

static STDOUT: Mutex<Stdout> = Mutex::new(Stdout);

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let uart = UART.lock();
        for c in s.chars() {
            uart.write(c as u8);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
