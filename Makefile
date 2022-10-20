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

ifeq ($(arch), x86_64)
	assembly_ext := asm
else
	assembly_ext := S
endif

assembly_source_files := $(wildcard src/arch/$(arch)/*.$(assembly_ext))
assembly_object_files := $(patsubst src/arch/$(arch)/%.$(assembly_ext), build/arch/$(arch)/%.o, $(assembly_source_files))

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

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.$(assembly_ext)
	@mkdir -p $(shell dirname $@)
ifeq ($(arch), x86_64)
	@nasm -Wall -felf64 $< -o $@
else
	@$(toolchain_prefix)as -c $< -o $@
endif

$(iso): $(kernel) $(grub_cfg)
	@$(eval TMP := $(shell mktemp -d))
	@mkdir -p $(TMP)/boot/grub
	@cp $(kernel) $(TMP)/boot/kernel.bin
	@cp $(grub_cfg) $(TMP)/boot/grub
	@grub2-mkrescue -o $(iso) $(TMP)
	@rm -r $(TMP)

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@$(toolchain_prefix)$(linker) --nmagic -z noexecstack --script=$(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)
