# To keep this in the first portion of the binary.
.section ".text.boot"

.global _start

_start:
    # disable interrupt
    csrw    mie, zero

    # zero bss
    la      t0, __bss_start
    la      t1, __bss_end_exclusive
.zero_bss:
    sw      zero, 0(t0)
    addi    t0, t0, 4
    blt     t0, t1, .zero_bss

    # set stack pointer
    la      sp, __boot_core_stack_end_exclusive

    # jump to Rust code, should not return
    j       kernel_main

    # failsafe
.loop:
    j       .loop
