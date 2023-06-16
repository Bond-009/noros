mod clk;
pub mod mmio;

use core::hint;
use core::arch::asm;
use core::fmt::{Arguments, Result, Write};

use crate::prelude::*;

use self::clk::init_clock;
use self::mmio::{read32, write32};

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

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_clock();

    init_uart();

    println!("Hello World!");

    loop { }
}

fn init_uart()
{
    let mut addr;
    let mut val;

    /* Config GPIOB8 and GPIOB9 to txd0 and rxd0 */
    addr = 0x02000030 + 0x04;
    val = read32(addr);
    val &= !(0xf << ((8 & 0x7) << 2));
    val |= (0x6 & 0xf) << ((8 & 0x7) << 2);
    write32(addr, val);

    val = read32(addr);
    val &= !(0xf << ((9 & 0x7) << 2));
    val |= (0x6 & 0xf) << ((9 & 0x7) << 2);
    write32(addr, val);

    /* Open the clock gate for uart0 */
    addr = 0x0200190c;
    val = read32(addr);
    val |= 1 << 0;
    write32(addr, val);

    /* Deassert uart0 reset */
    addr = 0x0200190c;
    val = read32(addr);
    val |= 1 << 16;
    write32(addr, val);

    /* Config uart0 to 115200-8-1-0 */
    addr = 0x02500000;
    write32(addr + 0x04, 0x0);
    write32(addr + 0x08, 0xf7);
    write32(addr + 0x10, 0x0);
    val = read32(addr + 0x0c);
    val |= 1 << 7;
    write32(addr + 0x0c, val);
    write32(addr + 0x00, 0xd & 0xff);
    write32(addr + 0x04, (0xd >> 8) & 0xff);
    val = read32(addr + 0x0c);
    val &= !(1 << 7);
    write32(addr + 0x0c, val);
    val = read32(addr + 0x0c);
    val &= !0x1f;
    val |= (0x3 << 0) | (0 << 2) | (0x0 << 3);
    write32(addr + 0x0c, val);
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
                self.write_char('\r')?;
            }

            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        let addr = 0x02500000;

        while (read32(addr + 0x7c) & (0x1 << 1)) == 0 {
            hint::spin_loop();
        }

        write32(addr + 0x00, c as u32);
        Ok(())
    }

    fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> Result {
        // Something goes wrong here...
        core::fmt::write(&mut self, args)
    }
}

fn counter() -> u64 {
    let value: u64;
    unsafe { asm!("csrr {}, time", out(reg) value, options(nomem, nostack)) };
    value
}

pub fn sdelay(us: u64) {
    let mut t1 = counter();
    let t2 = t1 + us * 24;
    while t2 >= t1 {
        t1 = counter();
    }
}
