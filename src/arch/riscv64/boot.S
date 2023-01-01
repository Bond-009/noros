.section ".text"

.global _start

_start:
    la      sp, __boot_core_stack_end_exclusive

    # jump to Rust code, should not return
    j kernel_main
    # failsafe
.loop:
    j .loop
