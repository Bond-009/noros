#![no_std]
#![allow(dead_code)]

mod arch;
mod drivers;
mod prelude;
mod sync;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::arch::_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => (eprint!("\n"));
    ($($arg:tt)*) => (eprint!("{}\n", format_args!($($arg)*)));
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
    loop {}
}
