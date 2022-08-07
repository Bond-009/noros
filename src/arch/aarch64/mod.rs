use core::arch::asm;
use core::fmt::{Arguments, Result, Write};
use core::hint;

use crate::prelude::*;

#[doc(hidden)]
pub fn _print(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() }
}

#[doc(hidden)]
pub fn _eprint(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() }
}

// TODO:
static mut WRITER: Uart = Uart::new();

// TODO:
static mut MMIO_BASE: *mut u32 = 0 as *mut _;

#[repr(transparent)]
struct MmioReg {
    v: usize
}

impl MmioReg {
    const unsafe fn new(val: usize) -> Self {
        Self { v: val }
    }

    fn as_ptr(&self) -> *const u32 {
        unsafe { (MMIO_BASE as usize + self.v) as *const _}
    }

    fn as_mut_ptr(&self) -> *mut u32 {
        unsafe { (MMIO_BASE as usize + self.v) as *mut _}
    }

    fn read(&self) -> u32 {
        unsafe { self.as_ptr().read_volatile() }
    }

    fn write(&self, val: u32) {
        unsafe { self.as_mut_ptr().write_volatile(val) }
    }
}

// The offsets for each register.
const GPIO_BASE: usize = 0x200000;
const GPFSEL1: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x04) };

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x94) };

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x98) };

/// Auxiliary Interrupt status
const AUX_IRQ: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15000) };

/// Auxiliary enables
const AUX_ENABLES: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15004) };

/// Mini Uart I/O Data
const AUX_MU_IO: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15040) };

/// Mini Uart Interrupt Enable
const AUX_MU_IER: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15044) };

/// Mini Uart Interrupt Identify
const AUX_MU_IIR: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15048) };

/// Mini Uart Line Control
const AUX_MU_LCR: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x1504C) };

/// Mini Uart Modem Control
const AUX_MU_MCR: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15050) };

/// Mini Uart Line Status
const AUX_MU_LSR: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15054) };

/// Mini Uart Modem Status
const AUX_MU_MSR: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15058) };

/// Mini Uart Scratch
const AUX_MU_SCRATCH: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x1505C) };

/// Mini Uart Extra Control
const AUX_MU_CNTL: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15060) };

/// Mini Uart Extra Status
const AUX_MU_STAT: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15064) };

/// Mini Uart Baudrate
const AUX_MU_BAUD: MmioReg = unsafe { MmioReg::new(GPIO_BASE + 0x15068) };

fn writec(val: u8) {
    while (AUX_MU_LSR.read() & 0x20) == 0 {
        // REVIEW:
        hint::spin_loop();
    }

    AUX_MU_IO.write(val as u32)
}

fn init_uart() {
    AUX_ENABLES.write(AUX_ENABLES.read() | 1);
    AUX_MU_CNTL.write(0);
    AUX_MU_LCR.write(3);
    AUX_MU_MCR.write(0);
    AUX_MU_IER.write(0);
    AUX_MU_IIR.write(0xc6);
    AUX_MU_BAUD.write(270);
    let mut r = GPFSEL1.read();
    r&=!((7<<12)|(7<<15)); // gpio14, gpio15
    r|=(2<<12)|(2<<15);    // alt5
    GPFSEL1.write(r);
    GPPUD.write(0);
    for _ in 0..150 {
        unsafe { asm!("nop") };
    }
    GPPUDCLK0.write((1<<14)|(1<<15));
    for _ in 0..150 {
        unsafe { asm!("nop") };
    }
    GPPUDCLK0.write(0);
    AUX_MU_CNTL.write(3);
}

#[derive(Debug)]
struct Uart;

impl Uart {
    const fn new() -> Self {
        Self {}
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            if c == '\n' {
                writec(b'\r');
            }

            writec(c as u8);
        }

        Ok(())
    }
}

#[no_mangle]
pub extern fn kernel_main(_dtb_ptr32: u64, _x1: u64, _x2: u64, _x3: u64) {
    // TODO:

    let reg: u32;
    unsafe { asm!("mrs {:x}, midr_el1", out(reg) reg) }
    let part_num = (reg >> 4) & 0xFFF;
    unsafe {
        match part_num {
            0xC07 => MMIO_BASE = 0x3F000000 as *mut _,
            0xD03 => MMIO_BASE = 0x3F000000 as *mut _,
            0xD08 => MMIO_BASE = 0xFE000000 as *mut _,
            _ => MMIO_BASE = 0x3F000000  as *mut _
        }
    }

    init_uart();

    println!("Hello World!");

    loop { }
}
