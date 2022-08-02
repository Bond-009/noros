use crate::{sync::mutex::Mutex, lazy::OnceCell};

// TODO:

static GDT: Mutex<OnceCell<GlobalDescriptorTable>> = Mutex::new(OnceCell::new());

#[repr(C, packed)]
struct TaskStateSegment {
    _reserved: u32,
    privilege_stack_table: [u64; 3],
    _reserved2: u64,
    interrupt_stack_table: [u64; 7],
    _reserved3: u64,
    _reserved4: u16,
    io_map_base_addr: u16
}

struct GlobalDescriptorTable {
}
