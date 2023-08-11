use core::fmt::{Result, Write};
use core::hint;
use core::mem::size_of;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::io::*;

#[cfg(target_arch = "x86_64")]
type Reg = u8;
#[cfg(not(target_arch = "x86_64"))]
type Reg = u32;

unsafe fn read(addr: usize) -> Reg {
    #[cfg(target_arch = "x86_64")]
    let r = inb(addr as u16);

    #[cfg(not(target_arch = "x86_64"))]
    let r = (addr as *const Reg).read_volatile();

    r
}

unsafe fn write(addr: usize, val: Reg) {
    #[cfg(target_arch = "x86_64")]
    outb(addr as u16, val);
    #[cfg(not(target_arch = "x86_64"))]
    (addr as *mut Reg).write_volatile(val)
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

    fn lsr(&self) -> Reg {
        unsafe { read(self.base + (size_of::<Reg>() * 5)) }
    }

    unsafe fn write(&mut self, c: Reg) {
        while (self.lsr() & (1 << 5)) == 0 {
            hint::spin_loop();
        }

        write(self.base, c);
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
        unsafe { self.write(c as Reg); }
        Ok(())
    }
}
