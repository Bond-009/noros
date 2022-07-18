.extern double_fault_handler
.extern page_fault_handler

.global double_fault
.global page_fault

.section .text

# TODO: save and restore registers
double_fault:
    call double_fault_handler

page_fault:
    pop rsi
    mov rdi, rsp
    call page_fault_handler
    iretq
