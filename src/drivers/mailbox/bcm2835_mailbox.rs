use core::{hint, marker::PhantomData};

use crate::arch::aarch64::mmio::MmioReg;

#[repr(C, align(16))]
pub struct MailboxBuffer<const N: usize>([u32; N]);

impl<const N: usize> From<[u32; N]> for MailboxBuffer<N> {
    fn from(v: [u32; N]) -> MailboxBuffer<N> {
        MailboxBuffer(v)
    }
}

const MAIL_BASE: usize = 0xB880;
const MBOX_READ: MmioReg = unsafe { MmioReg::new(MAIL_BASE) };
const MBOX_POLL: MmioReg = unsafe { MmioReg::new(MAIL_BASE + 0x10) };
const MBOX_SENDER: MmioReg = unsafe { MmioReg::new(MAIL_BASE + 0x14) };
const MBOX_STATUS: MmioReg = unsafe { MmioReg::new(MAIL_BASE + 0x18) };
const MBOX_CONFIG: MmioReg = unsafe { MmioReg::new(MAIL_BASE + 0x1C) };
const MBOX_WRITE: MmioReg = unsafe { MmioReg::new(MAIL_BASE + 0x20) };

const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

pub const MBOX_REQUEST: u32 = 0;

pub const MBOX_TAG_GETSERIAL: u32 = 0x10004;
pub const MBOX_TAG_LAST: u32 = 0;

#[repr(u8)]
pub enum Channel {
    PowerManagement = 0,
    FrameBuffer = 1,
    VirtualUART = 2,
    VCHIQ = 3,
    LEDs = 4,
    Buttons = 5,
    TouchScreen = 6,
    Counter = 7,
    PropertyTagsARMToVC = 8,
    PropertyTagsVCToARM = 9
}

#[repr(transparent)]
pub struct Message<'a> {
    v: u32,
    _lifetime: PhantomData<&'a ()>
}

impl<'a> Message<'a> {
    pub fn new<const N: usize>(buffer: &'a mut MailboxBuffer<N>, channel: Channel) -> Self {
        // Make sure the address fits in 32 bits
        debug_assert_eq!(buffer as *const _ as usize, buffer as *const _ as u32 as usize);

        // Make sure the address is aligned correctly
        debug_assert_eq!(buffer as *const _ as u8 & 0xF, 0);

        Self {
            v: buffer as *const _ as u32 | channel as u32,
            _lifetime: PhantomData
        }
    }
}

// TODO:
pub fn mbox_call(msg: Message) -> Result<(), ()> {
    while MBOX_STATUS.read() & MBOX_FULL != 0 {
        hint::spin_loop();
    }

    MBOX_WRITE.write(msg.v);

    loop {
        while MBOX_STATUS.read() & MBOX_EMPTY != 0 {
            hint::spin_loop();
        }

        if MBOX_READ.read() == msg.v {
            return Ok(())
        }
    }
}
