mod gdt;
mod interrupt;
pub mod io;

use core::cell::OnceCell;
use core::fmt::{Arguments, Write};
use core::ptr;

use crate::drivers::serial::ns16550::NS16550;
use crate::drivers::video::console::vga::{Writer, ScreenChar, Color};
use crate::sync::mutex::Mutex;
use crate::prelude::*;

// TODO: replace once lazy type is stabilized
static WRITER: Mutex<OnceCell<Writer>> = Mutex::new(OnceCell::new());

#[doc(hidden)]
pub fn _print(args: Arguments) {
    let mut lock = WRITER.lock();
    lock.get_mut().unwrap().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _eprint(args: Arguments) {
    let mut lock = WRITER.lock();
    let writer = lock.get_mut().unwrap();
    let current_color = writer.color_code();
    writer.set_fg_color(Color::Red);
    writer.write_fmt(args).unwrap();
    writer.set_color_code(current_color);
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    let mut writer = Writer::new(
        unsafe { &mut *ptr::slice_from_raw_parts_mut(0xb8000 as *mut ScreenChar, 80 * 25) },
        25,
        80);
    writer.clear(); // Clear screen

    WRITER.lock().set(writer.into()).unwrap();

    println!("Hello World!");

    interrupt::init_idt();

    println!("Interrupts set up");

    let mut w = NS16550::new(0x3F8);
    w.write_str("Hello COM1!\n").unwrap();

    loop {}
}
