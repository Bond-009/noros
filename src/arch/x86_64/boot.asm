bits 32

PAGE_PRESENT equ 1 << 0
PAGE_WRITABLE equ 1 << 1
PAGE_USER_ACCESSIBLE equ 1 << 2
PAGE_WRITE_TROUGH equ 1 << 3
PAGE_DISABLE_CACHE equ 1 << 4
PAGE_ACCESED equ 1 << 5
PAGE_DIRTY equ 1 << 6
PAGE_HUGE equ 1 << 7
PAGE_GLOBAL equ 1 << 8
PAGE_NO_EXECUTE equ 1 << 63

GDT_CONFORMING equ 1 << 42
GDT_EXECUTABLE equ 1 << 43
GDT_DESCRIPTOR_TYPE equ 1 << 44
GDT_PRESENT equ 1 << 47
GDT_64_BIT equ 1 << 53
GDT_32_BIT equ 1 << 54

global start
extern long_mode_start

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq GDT_64_BIT | GDT_PRESENT | GDT_DESCRIPTOR_TYPE | GDT_EXECUTABLE
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text
start:
    mov esp, stack_top

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables
    call enable_paging

    ; load the 64-bit GDT
    lgdt [gdt64.pointer]

    jmp gdt64.code:long_mode_start

    ; this should be unreachable now, leave it here just in case
    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    jmp halt

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, '0'
    jmp error

check_cpuid:
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ; in the FLAGS register. If we can flip it, CPUID is available.

    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, '1'
    jmp error

check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, '2'
    jmp error

; Prints `ERR: ` and the given error code to screen and hangs.
; parameter: error code (in ascii) in al
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    jmp halt

set_up_page_tables:
    ; map first PML4 entry to PDP table
    mov eax, pdp_table
    or eax, PAGE_PRESENT | PAGE_WRITABLE
    mov [pml4_table], eax

    ; map first PDP entry to PD table
    mov eax, pd_table
    or eax, PAGE_PRESENT | PAGE_WRITABLE
    mov [pdp_table], eax

    xor ecx, ecx         ; counter variable

.map_pdp_table:
    ; map ecx-th PD entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page
    or eax, PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE
    mov [pdp_table + ecx * 8], eax ; map ecx-th entry

    inc ecx            ; increase counter
    cmp ecx, 512       ; if counter == 512, the whole PD table is mapped
    jne .map_pdp_table  ; else map the next entry

    ret

enable_paging:
    ; load PML4 to cr3 register (cpu uses this to access the PML4 table)
    mov eax, pml4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

halt:
    hlt
    jmp halt

section .bss
align 4096
pml4_table:
    resb 4096
pdp_table:
    resb 4096
pd_table:
    resb 4096
stack_bottom:
    resb 64
stack_top:
