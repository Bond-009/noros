use crate::arch::aarch64::mmio::MmioReg;

const GPIO_BASE: usize = 0x200000;

pub const GPFSEL1: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x04) };

/// Controls actuation of pull up/down to ALL GPIO pins.
pub const GPPUD: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x94) };

/// Controls actuation of pull up/down for specific GPIO pin.
pub const GPPUDCLK0: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x98) };
