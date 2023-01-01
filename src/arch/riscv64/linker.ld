SECTIONS
{
    . = 0x0;
    .text     : { *(.text) }
    .boot_core_stack (NOLOAD) : ALIGN(16)
    {
        . += 0x1000;
        __boot_core_stack_end_exclusive = .;
    }
    /DISCARD/ : { *(.dynstr*) }
    /DISCARD/ : { *(.dynamic*) }
    /DISCARD/ : { *(.plt*) }
    /DISCARD/ : { *(.interp*) }
    /DISCARD/ : { *(.gnu*) }
    /DISCARD/ : { *(.note*) }
}
