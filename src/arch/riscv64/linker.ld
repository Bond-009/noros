ENTRY(_start)

STACK_SIZE = 0x100000;

SECTIONS
{
    . = 0;
    .text :
    {
        KEEP(*(.text.boot))
        *(.text.kernel_main)
        *(.text*)
    }
    .rodata : ALIGN(8) { *(.rodata*) }
    .data : ALIGN(8) { *(.data*) }
    .bss (NOLOAD) : ALIGN(8)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(8);
        __bss_end_exclusive = .;
    }
    .boot_core_stack (NOLOAD) : ALIGN(16)
    {
        . += STACK_SIZE;
        . = ALIGN(16);
        __boot_core_stack_end_exclusive = .;
    }

   /DISCARD/ : { *(.comment) }
}
