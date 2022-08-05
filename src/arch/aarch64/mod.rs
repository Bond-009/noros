use core::arch::asm;
use core::fmt::Arguments;
use core::ptr;

#[doc(hidden)]
pub fn _print(_args: Arguments) {
    write("print")
}

#[doc(hidden)]
pub fn _eprint(_args: Arguments) {
    write("error")
}

#[repr(align(16))]
struct MboxMessage([u32; 9]);

impl Default for MboxMessage {
    fn default() -> Self {
        Self([9 * 4, 0, 0x38002, 12, 8, 2, 3000000, 0, 0])
    }
}

// raspi2 and raspi3 have peripheral base address 0x3F000000,
// but raspi1 has peripheral base address 0x20000000. Ensure
// you are using the correct peripheral address for your
// hardware.

static mut MMIO_BASE: u32 = 0;

// The offsets for each register.
const GPIO_BASE: u32 = 0x200000;

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD: u32 = GPIO_BASE + 0x94;

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

// The base address for UART.
const UART0_BASE:u32 = GPIO_BASE + 0x1000;

// The offsets for reach register for the UART.
const UART0_DR: u32 = UART0_BASE + 0x00;
const UART0_RSRECR: u32 = UART0_BASE + 0x04;
const UART0_FR: u32 = UART0_BASE + 0x18;
const UART0_ILPR: u32 = UART0_BASE + 0x20;
const UART0_IBRD: u32 = UART0_BASE + 0x24;
const UART0_FBRD: u32 = UART0_BASE + 0x28;
const UART0_LCRH: u32 = UART0_BASE + 0x2C;
const UART0_CR: u32 = UART0_BASE + 0x30;
const UART0_IFLS: u32 = UART0_BASE + 0x34;
const UART0_IMSC: u32 = UART0_BASE + 0x38;
const UART0_RIS: u32 = UART0_BASE + 0x3C;
const UART0_MIS: u32 = UART0_BASE + 0x40;
const UART0_ICR: u32 = UART0_BASE + 0x44;
const UART0_DMACR: u32 = UART0_BASE + 0x48;
const UART0_ITCR: u32 = UART0_BASE + 0x80;
const UART0_ITIP: u32 = UART0_BASE + 0x84;
const UART0_ITOP: u32 = UART0_BASE + 0x88;
const UART0_TDR: u32 = UART0_BASE + 0x8C;

// The offsets for Mailbox registers
const MBOX_BASE: u32 = 0xB880;
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_STATUS: u32 = MBOX_BASE + 0x18;
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;

#[inline]
fn delay(count: i32) {
    unsafe {
        asm!(
            "0:",
            "subs {0:x}, {0:x}, #1",
            "b.ne 0b",
            inout(reg) count => _);
    }
}

fn mmio_write(reg: u32, val: u32) {
    unsafe { ptr::write_volatile((MMIO_BASE + reg) as *mut u32, val) };
}

fn mmio_read(reg: u32) -> u32 {
    unsafe { ptr::read_volatile((MMIO_BASE + reg) as *const u32,) }
}

fn transmit_fifo_full() -> bool {
    mmio_read(UART0_FR) & (1 << 5) > 0
}

fn receive_fifo_empty() -> bool {
    mmio_read(UART0_FR) & (1 << 4) > 0
}

fn writec(c: u8) {
    while transmit_fifo_full() {}
    mmio_write(UART0_DR, c as u32);
}

fn getc() -> u8 {
    while receive_fifo_empty() {}
    mmio_read(UART0_DR) as u8
}

fn write(msg: &str) {
    for c in msg.chars() {
        writec(c as u8)
    }
}

fn uart_init()
{
    // Disable UART0.
    mmio_write(UART0_CR, 0x00000000);
    // Setup the GPIO pin 14 && 15.

    // Disable pull up/down for all GPIO pins & delay for 150 cycles.
    mmio_write(GPPUD, 0x00000000);
    delay(150);

    // Disable pull up/down for pin 14,15 & delay for 150 cycles.
    mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
    delay(150);

    // Write 0 to GPPUDCLK0 to make it take effect.
    mmio_write(GPPUDCLK0, 0x00000000);

    // Clear pending interrupts.
    mmio_write(UART0_ICR, 0x7FF);

    // Set integer & fractional part of baud rate.
    // Divider = UART_CLOCK/(16 * Baud)
    // Fraction part register = (Fractional part * 64) + 0.5
    // Baud = 115200.

    // For Raspi3 and 4 the UART_CLOCK is system-clock dependent by default.
    // Set it to 3Mhz so that we can consistently set the baud rate

    // UART_CLOCK = 30000000;
    let mbox = MboxMessage::default();
    let r: u32 = ((&mbox) as *const _ as u32 & !0xF) | 8;
    // wait until we can talk to the VC
    while (mmio_read(MBOX_STATUS) & 0x80000000) > 0 { }
    // send our message to property channel and wait for the response
    mmio_write(MBOX_WRITE, r);
    while (mmio_read(MBOX_STATUS) & 0x40000000) > 0 || mmio_read(MBOX_READ) != r { }

    // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
    mmio_write(UART0_IBRD, 1);
    // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
    mmio_write(UART0_FBRD, 40);

    // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
    mmio_write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));

    // Mask all interrupts.
    mmio_write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                           (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));

    // Enable UART0, receive & transfer part of UART.
    mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
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
    loop {
        write("Hello Rust Kernel world!\n");
    }
/*
    loop {
        writec(getc())
    }*/
}
