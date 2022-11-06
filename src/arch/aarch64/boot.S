// To keep this in the first portion of the binary.
.section ".text.boot"

.global _start

_start:
    // read cpu id, stop slave cores
    mrs     x0, mpidr_el1
    and     x0, x0, #3
    cbnz    x0, .halt   // cpu id != 0, halt

    // check the current Exception Level
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100 // EL, bits [3:2]

    cmp     x0, #0b1000
    b.eq    .el2

    cmp     x0, #0b0100
    b.eq    .el1

.el3:
    msr     sctlr_el2, xzr
    msr     hcr_el2, xzr

    mrs     x0, scr_el3
    orr     x0, x0, #(1 << 10) // RW, bit [10]: Execution state control for lower Exception levels. (0: AArch32, 1: AArch64)
    orr     x0, x0, #1 // NS, bit [0]: Non-secure bit
    msr     scr_el3, x0
    mov     x0, #0b01001 // EL2h
    msr     spsr_el3, x0

    // set EL2 entry point
    adr     x0, .el2
    msr     elr_el3, x0

    eret

.el2:
    // initialize sctlr_el1 before entering EL1
    msr     sctlr_el1, xzr

    mrs     x0, hcr_el2
    orr     x0, x0, #(1 << 31) // RW, bit [31]; Execution state control for lower Exception levels. (0: AArch32, 1: AArch64)
    msr     hcr_el2, x0
    mov     x0, #0b1111000101 // EL1h
    msr     spsr_el2, x0

    // set EL1 entry point
    adr     x0, .el1
    msr     elr_el2, x0

    eret

.el1:
    // zero bss
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end_exclusive
.zero_bss:
    cmp     x0, x1
    b.eq   .set_sp
    stp     xzr, xzr, [x0], #16
    b       .zero_bss

.set_sp:
    // set top of stack just before our code (stack grows to a lower address per AAPCS64)
    ldr     x0, =__boot_core_stack_end_exclusive
    mov     sp, x0

    // jump to Rust code, should not return
    b      kernel_main
    // for failsafe, halt this core too
.halt:
    wfe
    b       .halt
