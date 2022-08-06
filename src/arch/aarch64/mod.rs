use core::arch::asm;
use core::fmt::Arguments;
use core::{ptr, hint};

#[doc(hidden)]
pub fn _print(_args: Arguments) {
    write("print")
}

#[doc(hidden)]
pub fn _eprint(_args: Arguments) {
    write("error")
}

// raspi2 and raspi3 have peripheral base address 0x3F000000,
// but raspi1 has peripheral base address 0x20000000. Ensure
// you are using the correct peripheral address for your
// hardware.

static mut MMIO_BASE: u32 = 0;

// The offsets for each register.
const GPIO_BASE: u32 = 0x200000;
const GPFSEL1: u32 = GPIO_BASE + 0x4;

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD: u32 = GPIO_BASE + 0x94;

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

const AUX_ENABLE: u32 = GPIO_BASE + 0x15004;
const AUX_MU_IO: u32 = GPIO_BASE + 0x15040;
const AUX_MU_IER: u32 = GPIO_BASE + 0x15044;
const AUX_MU_IIR: u32 = GPIO_BASE + 0x15048;
const AUX_MU_LCR: u32 = GPIO_BASE + 0x1504C;
const AUX_MU_MCR: u32 = GPIO_BASE + 0x15050;
const AUX_MU_LSR: u32 = GPIO_BASE + 0x15054;
const AUX_MU_MSR: u32 = GPIO_BASE + 0x15058;
const AUX_MU_SCRATCH: u32 = GPIO_BASE + 0x1505C;
const AUX_MU_CNTL: u32 = GPIO_BASE + 0x15060;
const AUX_MU_STAT: u32 = GPIO_BASE + 0x15064;
const AUX_MU_BAUD: u32 = GPIO_BASE + 0x15068;

// The base address for UART.
const UART0_BASE:u32 = GPIO_BASE + 0x1000;

// The offsets for reach register for the UART.

/// Data Register
const UART0_DR: u32 = UART0_BASE + 0x00;

/// Receive status register/error clear register
const UART0_RSRECR: u32 = UART0_BASE + 0x04;

/// Flag register
const UART0_FR: u32 = UART0_BASE + 0x18;

/// not in use
const UART0_ILPR: u32 = UART0_BASE + 0x20;

/// Integer Baud rate divisor
const UART0_IBRD: u32 = UART0_BASE + 0x24;

/// Fractional Baud rate divisor
const UART0_FBRD: u32 = UART0_BASE + 0x28;

/// Line Control register
const UART0_LCRH: u32 = UART0_BASE + 0x2C;

/// Control register
const UART0_CR: u32 = UART0_BASE + 0x30;

/// Interupt FIFO Level Select Register
const UART0_IFLS: u32 = UART0_BASE + 0x34;

/// Interupt Mask Set Clear Register
const UART0_IMSC: u32 = UART0_BASE + 0x38;

/// Raw Interupt Status Register
const UART0_RIS: u32 = UART0_BASE + 0x3C;

/// Masked Interupt Status Register
const UART0_MIS: u32 = UART0_BASE + 0x40;

/// Interupt Clear Register
const UART0_ICR: u32 = UART0_BASE + 0x44;

/// DMA Control Register
const UART0_DMACR: u32 = UART0_BASE + 0x48;

/// Test Control register
const UART0_ITCR: u32 = UART0_BASE + 0x80;

/// Integration test input reg
const UART0_ITIP: u32 = UART0_BASE + 0x84;

/// Integration test output reg
const UART0_ITOP: u32 = UART0_BASE + 0x88;

/// Test Data reg
const UART0_TDR: u32 = UART0_BASE + 0x8C;

// The offsets for Mailbox registers
const MBOX_BASE: u32 = 0xB880;
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_STATUS: u32 = MBOX_BASE + 0x18;
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;
/*
fn transmit_fifo_full() -> bool {
    mmio_read(UART0_FR) & (1 << 5) > 0
}

fn receive_fifo_empty() -> bool {
    mmio_read(UART0_FR) & (1 << 4) > 0
}

fn transmit_fifo_busy() -> bool {
    mmio_read(UART0_FR) & (1 << 3) > 0
}

fn writec(c: u8) {
    while transmit_fifo_full() {
        hint::spin_loop();
    }
    mmio_write(UART0_DR, c as u32);
}

fn getc() -> u8 {
    while receive_fifo_empty() {
        hint::spin_loop();
    }
    mmio_read(UART0_DR) as u8
}

fn uart_init()
{
    // 1) Disable UART0.
    mmio_write(UART0_CR, 0);

    // 2) Wait for the end of transmission or reception of the current character
    while transmit_fifo_busy() {
        hint::spin_loop();
    }

    // 3) Flush the transmit FIFO by setting the FEN bit to 0
    mmio_write(UART0_LCRH, mmio_read(UART0_LCRH) & !(1 << 4));


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
    mmio_write(UART0_ICR, 0);

    // Set integer & fractional part of baud rate.
    // Divider = UART_CLOCK/(16 * Baud)
    // Fraction part register = (Fractional part * 64) + 0.5
    // Baud = 115200.

    // For Raspi3 and 4 the UART_CLOCK is system-clock dependent by default.
    // Set it to 3Mhz so that we can consistently set the baud rate

    // UART_CLOCK = 30000000;
    let r: u32 = ((&MBOX) as *const _ as u32 & !0xF) | 8;
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

    // 4 & 5) Reprogram the Control Register, UART_CR and enable the UART
    // Enable UART0, receive & transfer part of UART.
    mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
}

*/

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

fn uart_init() {
    mmio_write(AUX_ENABLE, mmio_read(AUX_ENABLE) | 1);
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

fn writec(val: u8) {
    while (mmio_read(AUX_MU_LSR) & 0x20) == 0 {
        // REVIEW:
        hint::spin_loop();
    }

    mmio_write(AUX_MU_IO, val as u32)
}

fn write(msg: &str) {
    for c in msg.chars() {
        if c == '\n' {
            writec(b'\r')
        }
        writec(c as u8)
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

    write("Hello Rust Kernel world!\n");

    loop { }
}
