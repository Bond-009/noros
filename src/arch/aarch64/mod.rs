pub mod mmio;

use core::arch::{asm, global_asm};
use core::fmt::{Arguments, Result, Write};
use core::hint;

use crate::prelude::*;
use crate::drivers::gpio::bcm2835_gpio::*;
use crate::drivers::mailbox::bcm2835_mailbox::*;

use self::mmio::MmioReg;

global_asm!(include_str!("boot.S"));

#[doc(hidden)]
pub fn _print(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() };
}

#[doc(hidden)]
pub fn _eprint(args: Arguments) {
    unsafe { WRITER.write_fmt(args).unwrap() };
}

// TODO: make thread safe
static mut WRITER: Uart = Uart::new();

const AUX_BASE: usize = 0x215000;

/// Auxiliary Interrupt status
pub const AUX_IRQ: MmioReg = unsafe { MmioReg::new(AUX_BASE) };

/// Auxiliary enables
pub const AUX_ENABLES: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x4) };

/// Mini Uart I/O Data
pub const AUX_MU_IO: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x40) };

/// Mini Uart Interrupt Enable
pub const AUX_MU_IER: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x44) };

/// Mini Uart Interrupt Identify
pub const AUX_MU_IIR: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x48) };

/// Mini Uart Line Control
pub const AUX_MU_LCR: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x4C) };

/// Mini Uart Modem Control
pub const AUX_MU_MCR: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x50) };

/// Mini Uart Line Status
pub const AUX_MU_LSR: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x54) };

/// Mini Uart Modem Status
pub const AUX_MU_MSR: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x58) };

/// Mini Uart Scratch
pub const AUX_MU_SCRATCH: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x5C) };

/// Mini Uart Extra Control
pub const AUX_MU_CNTL: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x60) };

/// Mini Uart Extra Status
pub const AUX_MU_STAT: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x64) };

/// Mini Uart Baudrate
pub const AUX_MU_BAUD: MmioReg = unsafe { MmioReg::new(AUX_BASE + 0x68) };

fn writec(val: u8) {
    while (AUX_MU_LSR.read() & 0x20) == 0 {
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
    r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
    r |= (2 << 12) | (2 << 15); // alt5
    GPFSEL1.write(r);
    GPPUD.write(0);
    for _ in 0..150 {
        unsafe { asm!("nop") };
    }
    GPPUDCLK0.write((1 << 14) | (1 << 15));
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
        Self
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
pub extern fn kernel_main(_dtb_ptr32: u64, _x1: u64, _x2: u64, _x3: u64) -> ! {
    mmio::init();

    init_uart();
    println!("Hello World!");

    let mut mbox: MailboxBuffer<8> = [8 * 4, MBOX_REQUEST, MBOX_TAG_GETSERIAL, 8, 8, 0, 0, MBOX_TAG_LAST].into();
    mbox_call(Message::new(&mut mbox, Channel::PropertyTagsARMToVC)).unwrap();
    unsafe {
        let s1 = *(&mbox as *const _ as *const u32).offset(6);
        let s2 = *(&mbox as *const _ as *const u32).offset(5);
        println!("Serial number: {:X}{:X}", s1, s2);
    }

    loop { }
}
