.macro ADR_REL register, symbol
    adrp    \register, \symbol
    add     \register, \register, #:lo12:\symbol
.endm

// To keep this in the first portion of the binary.
.section ".text.boot"

.global _start

_start:
    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbnz    x1, .halt   // cpu id != 0, halt

    // zero bss
    ADR_REL     x0, __bss_start
    ADR_REL     x1, __bss_end_exclusive
.zero_bss:
    cmp     x0, x1
    b.eq   .jmp_to_rust
    stp     xzr, xzr, [x0], #16
    b       .zero_bss

    // jump to C code, should not return
.jmp_to_rust:
    // set top of stack just before our code (stack grows to a lower address per AAPCS64)
    ADR_REL     x0, __boot_core_stack_end_exclusive
    mov     sp, x0

    b      kernel_main
    // for failsafe, halt this core too
.halt:
    wfe
    b       .halt
