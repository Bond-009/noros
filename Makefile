arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/noros-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(assembly_source_files))
rust_os := target/$(arch)-unknown-none/debug/libnoros.a

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -rf build
	@cargo clean

run: $(iso)
	@qemu-system-$(arch) -cdrom $(iso)

iso: $(iso)

kernel:
	@cargo build --target $(arch)-unknown-none

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -Wall -felf64 $< -o $@
