ENTRY(_start)

SECTIONS
{
    . = 0;
    .boot_core_stack (NOLOAD) :
    {
        . += 0x80000;
        __boot_core_stack_end_exclusive = .;
    }
    .text :
    {
        KEEP(*(.text.boot))
        *(.text.kernel_main)
        *(.text*)
    }
    .rodata : ALIGN(8) { *(.rodata*) }
    .data : ALIGN(4096) { *(.data*) }
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(4096);
        __bss_end_exclusive = .;
    }

   /DISCARD/ : { *(.comment) }
}
