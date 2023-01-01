use core::fmt::Arguments;

#[doc(hidden)]
pub fn _print(_args: Arguments) {
    todo!();
}

#[doc(hidden)]
pub fn _eprint(_args: Arguments) {
    todo!();
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    loop { }
}
