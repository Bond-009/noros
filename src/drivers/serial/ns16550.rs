use core::fmt::{Result, Write};
use core::hint;
use core::mem::size_of;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::io::*;

#[cfg(target_arch = "x86_64")]
type Reg = u8;
#[cfg(not(target_arch = "x86_64"))]
type Reg = u32;

const RBR: usize = 0;
const THR: usize = 0;
const DLL: usize = 0;
const DLM: usize = size_of::<Reg>();
const IER: usize = size_of::<Reg>();
const FCR: usize = 2 * size_of::<Reg>();
const LCR: usize = 3 * size_of::<Reg>();
const MCR: usize = 4 * size_of::<Reg>();
const LSR: usize = 5 * size_of::<Reg>();

const FCR_FIFO_ENABLE: Reg = 1;
const FCR_RCVR_FIFO_RESET: Reg = 1 << 1;
const FCR_XMIT_FIFO_RESET: Reg = 1 << 2;

const FCR_DEFAULT_VAL: Reg = FCR_FIFO_ENABLE | FCR_RCVR_FIFO_RESET | FCR_XMIT_FIFO_RESET;

/// Word length select bit 0
const LCR_WLS0: Reg = 1;

/// Word length select bit 1
const LCR_WLS1: Reg = 1 << 1;

/// Number of stop bits
const LCR_STB: Reg = 1 << 2;

/// Parity enable
const LCR_PEN: Reg = 1 << 3;

/// Even parity select
const LCR_EPS: Reg = 1 << 4;

/// Divisor Latch Access Bit
const LCR_DLAB: Reg = 1 << 7;

/// Transmitter holding register
const LSR_THRE: Reg = 1 << 5;

macro_rules! read_registers {
    ($($name:ident: ($reg:ident),)*) => {
        $(
            unsafe fn $name(&self) -> Reg {
                #[cfg(target_arch = "x86_64")]
                let r = inb((self.base + $reg) as u16);

                #[cfg(not(target_arch = "x86_64"))]
                let r = ((self.base + $reg) as *const Reg).read_volatile();

                r
            }
        )*
    }
}

macro_rules! write_registers {
    ($($name:ident: ($reg:ident),)*) => {
        $(
            unsafe fn $name(&self, v: Reg) {
                #[cfg(target_arch = "x86_64")]
                outb((self.base + $reg) as u16, v);

                #[cfg(not(target_arch = "x86_64"))]
                ((self.base + $reg) as *mut Reg).write_volatile(v)
            }
        )*
    }
}

pub struct NS16550 {
    base: usize
}

impl NS16550 {
    pub const fn new(base: usize) -> Self {
        Self {
            base
        }
    }

    const fn calc_divisor(clock: u32, baud_rate: u32) -> u16 {
        (clock / (16 * baud_rate)) as u16
    }

    pub unsafe fn init(&self, clock: u32, baud_rate: u32) {
        self.set_fcr(FCR_DEFAULT_VAL);

        // TODO: don't hard code this
        self.set_lcr(LCR_WLS0 | LCR_WLS1); // 8 data bits, 1 stop bit, no parity
        self.set_ier(0);

        // Set baud rate
        let divisor = Self::calc_divisor(clock, baud_rate);
        self.set_lcr(self.lcr() | LCR_DLAB);
        self.set_dll(((divisor) as u8).into());
        self.set_dlm(((divisor >> 8) as u8).into());
        self.set_lcr(self.lcr() & !LCR_DLAB);
    }

    unsafe fn tx(&mut self, c: Reg) {
        while (self.lsr() & LSR_THRE) == 0 {
            hint::spin_loop();
        }

        self.set_thr(c);
    }

    read_registers! {
        rbr: (RBR),
        ier: (IER),
        lcr: (LCR),
        lsr: (LSR),
    }

    write_registers! {
        set_thr: (THR),
        set_dll: (DLL),
        set_dlm: (DLM),
        set_ier: (IER),
        set_fcr: (FCR),
        set_lcr: (LCR),
        set_lsr: (LSR),
    }
}

impl Write for NS16550 {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r')?;
            }

            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        unsafe { self.tx(c as Reg); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! calc_divisor_tests {
        ($($name:ident: ($clock:literal, $input:literal, $expected:literal),)*) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(NS16550::calc_divisor($clock, $input), $expected);
                }
            )*
        }
    }

    calc_divisor_tests! {
        calc_divisor_1843200_50: (1843200, 50, 2304),
        calc_divisor_1843200_300: (1843200, 300, 384),
        calc_divisor_1843200_1200: (1843200, 1200, 96),
        calc_divisor_1843200_2400: (1843200, 2400, 48),
        calc_divisor_1843200_4800: (1843200, 4800, 24),
        calc_divisor_1843200_9600: (1843200, 9600, 12),
        calc_divisor_1843200_19200: (1843200, 19200, 6),
        calc_divisor_1843200_38400: (1843200, 38400, 3),
        calc_divisor_1843200_57600: (1843200, 57600, 2),
        calc_divisor_1843200_115200: (1843200, 115200, 1),
        calc_divisor_24000000_115200: (24000000, 115200, 13),
    }
}
