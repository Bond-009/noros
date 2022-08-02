use core::arch::{asm, global_asm};
use core::mem;

use crate::lazy::OnceCell;
use crate::prelude::*;
use crate::sync::mutex::Mutex;

static IDT: Mutex<OnceCell<InterruptDescriptorTable>> = Mutex::new(OnceCell::new());

global_asm!(include_str!("_asm/interrupt.asm"));

extern "C" {
    fn double_fault() -> !;
    fn page_fault() -> !;
}

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum GateType {
    #[default]
    InterruptGate = 0xE,
    TrapGate = 0xF
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct IDTEntryOptions(u8);

impl IDTEntryOptions {
    pub fn gate_type(&self) -> GateType {
        match self.0 & 0x0F {
            0xE => GateType::InterruptGate,
            0xF => GateType::TrapGate,
            v => panic!("Invalid Gate Type: {}", v)
        }
    }

    pub fn set_gate_type(&mut self, gate_type: GateType) {
        self.0 = self.0 & 0xF0 | gate_type as u8
    }

    pub fn dpl(&self) -> u8 {
        self.0 & 0b01100000
    }

    pub fn set_dpl(&mut self, dpl: u8) {
        self.0 = self.0 & 0b10011111 | dpl << 5
    }

    pub fn present(&self) -> bool {
        (self.0 & 0b10000000) == 0b10000000
    }

    pub fn set_present(&mut self, present: bool) {
        self.0 = self.0 & 0x7F | ((present as u8) << 7)
    }
}

impl Default for IDTEntryOptions {
    fn default() -> Self {
        let mut ret = Self(0);
        ret.set_gate_type(Default::default());
        ret
    }
}

impl From<u8> for IDTEntryOptions {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct IDTEnrty {
    fn_ptr_low: u16,
    gdt_selector: u16,
    ist: u8,
    options: IDTEntryOptions,
    fn_ptr_mid: u16,
    fn_ptr_high: u32,
    _reserved: u32
}

impl IDTEnrty {
    pub fn new<T: Into<u64>>(handler: T) -> Self {
        let fn_ptr: u64 = handler.into();
        let mut cs: u16;
        unsafe { asm!("mov {:x}, cs", out(reg) cs); }
        let mut options = IDTEntryOptions::default();
        options.set_present(true);
        Self {
            fn_ptr_low: (fn_ptr & 0xFFFF) as u16,
            gdt_selector: cs,
            ist: 0,
            options,
            fn_ptr_mid: ((fn_ptr >> 16) & 0xFFFF) as u16,
            fn_ptr_high: (fn_ptr >> 32) as u32,
            _reserved: 0
        }
    }
}

#[repr(C, packed)]
struct InterruptDescriptorTablePtr {
    limit: u16,
    base: u64
}

#[derive(Default, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: IDTEnrty,
    pub debug: IDTEnrty,
    pub non_maskable_interrupt: IDTEnrty,
    pub breakpoint: IDTEnrty,
    pub overflow: IDTEnrty,
    pub bound_range_exceeded: IDTEnrty,
    pub invalid_opcode: IDTEnrty,
    pub device_not_available: IDTEnrty,
    pub double_fault: IDTEnrty,
    pub coprocesser_segment_overrun: IDTEnrty,
    pub invalid_tss: IDTEnrty,
    pub segment_not_present: IDTEnrty,
    pub stack_segment_fault: IDTEnrty,
    pub general_protection_fault: IDTEnrty,
    pub page_fault: IDTEnrty,
    _reserved: IDTEnrty,
    pub x87_floating_point_error: IDTEnrty,
    pub alignment_check: IDTEnrty,
    pub machine_check: IDTEnrty,
    pub simd_floating_point_exception: IDTEnrty,
    pub virtualization: IDTEnrty,
    _reserved_2: [IDTEnrty; 9],
    pub security_exception: IDTEnrty,
    _reserved_3: IDTEnrty,
    interrupts: Int,
}

// TODO: remove type when Default is implemented for all array sizes #88744
#[derive(Debug)]
#[repr(transparent)]
struct Int([IDTEnrty; 256 - 32]);

impl Default for Int {
    fn default() -> Self {
        Self([IDTEnrty::default(); mem::size_of::<Self>() / mem::size_of::<IDTEnrty>()])
    }
}

impl InterruptDescriptorTable {
    // REVIEW: unsafe
    pub fn load(&self) {
        let ptr = InterruptDescriptorTablePtr {
            limit: (mem::size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64
        };

        unsafe { asm!("lidt [{}]", in(reg) &ptr); }
    }
}

pub fn init_idt() {
    let mut idt_lock = IDT.lock();
    idt_lock.set(InterruptDescriptorTable::default()).unwrap();
    let mut idt = idt_lock.get_mut().unwrap();

    idt.double_fault = IDTEnrty::new(double_fault as u64);
    idt.page_fault = IDTEnrty::new(page_fault as u64);

    idt.load();
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

#[no_mangle]
extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, _error_code: u64) -> ! {
    debug_assert_eq!(_error_code, 0);
    panic!("EXCEPTION: double fault\n{:#?}", stack_frame);
}

// TODO: make type for error code
#[no_mangle]
extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    eprintln!("EXCEPTION: page fault, code: {}\n{:#?}", error_code, stack_frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_options_default() {
        assert_eq!(IDTEntryOptions::default().0, 0x0E);
    }

    #[test]
    fn entry_options_present() {
        let mut options = IDTEntryOptions::default();
        options.set_present(true);
        assert_eq!(options.0, 0x8E);
    }
}
