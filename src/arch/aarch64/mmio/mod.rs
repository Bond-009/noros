use core::arch::asm;

// TODO:
static mut MMIO_BASE: *mut u32 = 0 as *mut _;

// REVIEW:
pub fn init() {
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
}

#[repr(transparent)]
pub struct MmioReg {
    v: usize
}

// REVIEW: split read and write into different traits?
impl MmioReg {
    pub const unsafe fn new(val: usize) -> Self {
        Self { v: val }
    }

    pub fn as_ptr(&self) -> *const u32 {
        unsafe { (MMIO_BASE as usize + self.v) as *const _}
    }

    pub fn as_mut_ptr(&self) -> *mut u32 {
        unsafe { (MMIO_BASE as usize + self.v) as *mut _}
    }

    pub fn read(&self) -> u32 {
        unsafe { self.as_ptr().read_volatile() }
    }

    pub fn write(&self, val: u32) {
        unsafe { self.as_mut_ptr().write_volatile(val) }
    }
}
