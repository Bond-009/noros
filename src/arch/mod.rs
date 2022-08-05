#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::_print as _print;
#[cfg(target_arch = "aarch64")]
pub use aarch64::_eprint as _eprint;

#[cfg(target_arch = "x86_64")]
pub use x86_64::_print as _print;
#[cfg(target_arch = "x86_64")]
pub use x86_64::_eprint as _eprint;
