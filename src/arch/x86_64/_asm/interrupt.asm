.extern double_fault_handler
.extern page_fault_handler

.global double_fault
.global page_fault

.section .text

double_fault:
    pop rsi
    mov rdi, rsp
    call double_fault_handler

page_fault:
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11

    lea rdi, [rsp + 10 * 8] # load exception stack frame pointer into rdi
    mov rsi, [rsp + 9 * 8] # load error code into rsi
    sub rsp, 8 # align stack pointer
    call page_fault_handler
    add rsp, 8 # undo stack pointer alignment

    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    add rsp, 8

    iretq
