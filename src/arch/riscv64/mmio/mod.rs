pub fn write32(addr: usize, value: u32) {
    unsafe { (addr as *mut u32).write_volatile(value) }
}

pub fn read32(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}
