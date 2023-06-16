OUTPUT_ARCH(riscv)
ENTRY(_start)

STACK_SIZE = 0x100000;

MEMORY
{
    ram  : org = 0x40000000, len = 128M
}

SECTIONS
{
    .text :
    {
        KEEP(*(.text.boot))
        *(.text.kernel_main)
        *(.text*)
    } > ram
    .rodata ALIGN(8) :
    {
        *(.rodata*)
        *(.srodata*)
    } > ram
    .data ALIGN(8) :
    {
        *(.sdata*)
        *(.data*)
        . = ALIGN(8);
    } > ram
    .bss ALIGN(8) (NOLOAD) :
    {
        __bss_start = .;
        *(.bss*)
        *(.sbss*)
        . = ALIGN(8);
        __bss_end_exclusive = .;
    } > ram
    .boot_core_stack ALIGN(16) (NOLOAD) :
    {
        . += STACK_SIZE;
        . = ALIGN(16);
        __boot_core_stack_end_exclusive = .;
    } > ram

   /DISCARD/ : { *(.comment) }
}
