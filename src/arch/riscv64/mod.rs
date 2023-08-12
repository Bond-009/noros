mod clk;
pub mod mmio;

use core::arch::asm;
use core::fmt::{Arguments, Write};

use crate::drivers::serial::ns16550::NS16550;
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
static mut WRITER: NS16550 = NS16550::new(0x02500000);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_clock();
    init_jtag();
    init_uart();

    println!("Hello World!");

    loop { }
}

fn init_jtag()
{
    /* Config GPIOF0, GPIOF1, GPIOF3 and GPIOF5 to JTAG mode */
    let addr = 0x020000f0;
    let mut val = read32(addr);
    val &= !(0xf << ((0 & 0x7) << 2));
    val |= (0x4 & 0xf) << ((0 & 0x7) << 2);
    write32(addr, val);

    val = read32(addr);
    val &= !(0xf << ((1 & 0x7) << 2));
    val |= (0x4 & 0xf) << ((1 & 0x7) << 2);
    write32(addr, val);

    val = read32(addr);
    val &= !(0xf << ((3 & 0x7) << 2));
    val |= (0x4 & 0xf) << ((3 & 0x7) << 2);
    write32(addr, val);

    val = read32(addr);
    val &= !(0xf << ((5 & 0x7) << 2));
    val |= (0x4 & 0xf) << ((5 & 0x7) << 2);
    write32(addr, val);
}

fn init_uart()
{
    /* Config GPIOB8 and GPIOB9 to txd0 and rxd0 */
    let mut addr = 0x02000030 + 0x04;
    let mut val = read32(addr);
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

    unsafe { WRITER.init(24000000, 115200); }
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
