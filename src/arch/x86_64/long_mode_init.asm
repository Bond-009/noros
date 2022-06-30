bits 64

global long_mode_start

section .text
long_mode_start:
    ; load 0 into all data segment registers
    xor eax, eax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; call the rust main
    extern _start
    call _start

    hlt
