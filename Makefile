.ONESHELL:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

arch ?= x86_64
ifeq ($(arch), aarch64)
	target := $(arch)-unknown-none-softfloat
else
	target := $(arch)-unknown-none
endif

linker ?= ld
ifneq ($(arch), $(shell uname -m))
	ifeq ($(arch), aarch64)
		toolchain_prefix ?= aarch64-none-elf-
	endif
endif
# default to empty
toolchain_prefix ?=

kernel := build/kernel-$(arch).elf
iso := build/noros-$(arch).iso
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg

# we use nasm for x86_64 asmsembly and Rusts global_asm for all other platforms
ifeq ($(arch), x86_64)
	assembly_object_files := $(patsubst src/arch/x86_64/%.asm, build/arch/x86_64/%.o, $(wildcard src/arch/x86_64/*.asm))
else
	assembly_object_files :=
endif

rust_os := target/$(target)/debug/libnoros.a

.PHONY: clean test objdump run iso kernel

clean:
	@rm -rf build
	@cargo clean

test:
	@cargo test --target $(shell rustc -vV | sed -n 's/host: //p')

ifeq ($(arch), aarch64)
run: $(kernel)
	@$(toolchain_prefix)objcopy $(kernel) -O binary build/kernel8.img
	@qemu-system-$(arch) -machine raspi3b -serial null -serial stdio -kernel $(kernel) -display none -d int
else
run: $(iso)
	@qemu-system-$(arch) -monitor stdio -cdrom $(iso)
endif

objdump: $(kernel)
	@$(toolchain_prefix)objdump --disassemble --demangle $(kernel)

iso: $(iso)

kernel:
	@cargo build --target $(target)

# compile x86_64 assembly files
build/arch/x86_64/%.o: src/arch/x86_64/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -Wall -felf64 $< -o $@

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p build
	@$(toolchain_prefix)$(linker) --nmagic -z noexecstack --script=$(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)
