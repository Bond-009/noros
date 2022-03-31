section .multiboot_header

MAGIC equ 0xe85250d6 ; magic number (multiboot 2)
ARCH equ 0 ; architecture 0 (protected mode i386)
HEADER_LENGTH equ header_end - header_start
CHECKSUM equ -(MAGIC + ARCH + HEADER_LENGTH)

header_start:
    dd MAGIC
    dd ARCH
    dd HEADER_LENGTH
    dd CHECKSUM

    ; insert optional multiboot tags here

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
