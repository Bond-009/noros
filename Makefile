arch ?= x86_64
linker ?= ld

kernel := build/kernel-$(arch).bin
iso := build/noros-$(arch).iso
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg

ifeq ($(arch), x86_64)
	assembly_ext := asm
else
	assembly_ext := S
endif

assembly_source_files := $(wildcard src/arch/$(arch)/*.$(assembly_ext))
assembly_object_files := $(patsubst src/arch/$(arch)/%.$(assembly_ext), build/arch/$(arch)/%.o, $(assembly_source_files))

rust_os := target/$(arch)-unknown-none/debug/libnoros.a

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -rf build
	@cargo clean

ifeq ($(arch), aarch64)
run: $(kernel)
	@qemu-system-$(arch) -M raspi3b -serial stdio -kernel $(kernel)
else
run: $(iso)
	@qemu-system-$(arch) -cdrom $(iso)
endif

iso: $(iso)

kernel:
	@cargo build --target $(arch)-unknown-none

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.$(assembly_ext)
	@mkdir -p $(shell dirname $@)
ifeq ($(arch), aarch64)
	@aarch64-elf-as -c $< -o $@
else
	@nasm -Wall -felf64 $< -o $@
endif

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@$(linker) -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)
