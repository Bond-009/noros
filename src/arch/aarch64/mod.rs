use core::arch::asm;
use core::fmt::{Arguments, Result, Write};
use core::{ptr, hint};

use crate::prelude::*;

#[doc(hidden)]
pub fn _print(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() };
}

#[doc(hidden)]
pub fn _eprint(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() };
}

static mut WRITER: Uart = Uart::new();

// TODO:
static mut MMIO_BASE: u32 = 0;

// The offsets for each register.
const GPIO_BASE: u32 = 0x200000;
const GPFSEL1: u32 = GPIO_BASE + 0x4;

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD: u32 = GPIO_BASE + 0x94;

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

/// Auxiliary Interrupt status
const AUX_IRQ: u32 = GPIO_BASE + 0x15000;

/// Auxiliary enables
const AUX_ENABLES: u32 = GPIO_BASE + 0x15004;

/// Mini Uart I/O Data
const AUX_MU_IO: u32 = GPIO_BASE + 0x15040;

/// Mini Uart Interrupt Enable
const AUX_MU_IER: u32 = GPIO_BASE + 0x15044;

/// Mini Uart Interrupt Identify
const AUX_MU_IIR: u32 = GPIO_BASE + 0x15048;

/// Mini Uart Line Control
const AUX_MU_LCR: u32 = GPIO_BASE + 0x1504C;

/// Mini Uart Modem Control
const AUX_MU_MCR: u32 = GPIO_BASE + 0x15050;

/// Mini Uart Line Status
const AUX_MU_LSR: u32 = GPIO_BASE + 0x15054;

/// Mini Uart Modem Status
const AUX_MU_MSR: u32 = GPIO_BASE + 0x15058;

/// Mini Uart Scratch
const AUX_MU_SCRATCH: u32 = GPIO_BASE + 0x1505C;

/// Mini Uart Extra Control
const AUX_MU_CNTL: u32 = GPIO_BASE + 0x15060;

/// Mini Uart Extra Status
const AUX_MU_STAT: u32 = GPIO_BASE + 0x15064;

/// Mini Uart Baudrate
const AUX_MU_BAUD: u32 = GPIO_BASE + 0x15068;

fn mmio_write(reg: u32, val: u32) {
    unsafe { ptr::write_volatile((MMIO_BASE + reg) as *mut u32, val) };
}

fn mmio_read(reg: u32) -> u32 {
    unsafe { ptr::read_volatile((MMIO_BASE + reg) as *const u32,) }
}

fn writec(val: u8) {
    while (mmio_read(AUX_MU_LSR) & 0x20) == 0 {
        // REVIEW:
        hint::spin_loop();
    }

    mmio_write(AUX_MU_IO, val as u32)
}

fn uart_init() {
    mmio_write(AUX_ENABLES, mmio_read(AUX_ENABLES) | 1);
    mmio_write(AUX_MU_CNTL, 0);
    mmio_write(AUX_MU_LCR, 3);
    mmio_write(AUX_MU_MCR, 0);
    mmio_write(AUX_MU_IER, 0);
    mmio_write(AUX_MU_IIR, 0xc6);
    mmio_write(AUX_MU_BAUD, 270);
    let mut r = mmio_read(GPFSEL1);
    r&=!((7<<12)|(7<<15)); // gpio14, gpio15
    r|=(2<<12)|(2<<15);    // alt5
    mmio_write(GPFSEL1, r);
    mmio_write(GPPUD, 0);
    for _ in 0..150 {
        unsafe { asm!("nop") };
    }
    mmio_write(GPPUDCLK0, (1<<14)|(1<<15));
    for _ in 0..150 {
        unsafe { asm!("nop") };
    }
    mmio_write(GPPUDCLK0, 0);
    mmio_write(AUX_MU_CNTL, 3);
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
            0xC07 => MMIO_BASE = 0x3F000000,
            0xD03 => MMIO_BASE = 0x3F000000,
            0xD08 => MMIO_BASE = 0xFE000000,
            _ => MMIO_BASE = 0x3F000000
        }
    }

    uart_init();
    println!("Hello Rust Kernel world!");

    loop { }
}
