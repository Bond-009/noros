MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

arch ?= x86_64
ifeq ($(arch), aarch64)
	target := $(arch)-unknown-none-softfloat
else ifeq ($(arch), riscv64)
	target := $(arch)gc-unknown-none-elf
else
	target := $(arch)-unknown-none
endif

linker ?= ld
ifneq ($(arch), $(shell uname -m))
	toolchain_prefix ?= $(arch)-elf-
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

ifeq (, $(shell command -v grub2-mkrescue))
	grub_mkrescue := grub-mkrescue
else
	grub_mkrescue := grub2-mkrescue
endif

assembly_source_files := $(wildcard src/arch/$(arch)/*.$(assembly_ext))
assembly_object_files := $(patsubst src/arch/$(arch)/%.$(assembly_ext), build/arch/$(arch)/%.o, $(assembly_source_files))

rust_os := target/$(target)/debug/libnoros.a

.PHONY: clean test gdb objdump run iso kernel

clean:
	@rm -rf build
	@cargo clean

test:
	@cargo test --target $(shell rustc -vV | sed -n 's/host: //p')

objdump: $(kernel)
	@$(toolchain_prefix)objdump --disassemble --demangle $(kernel)

gdb:
	@RUST_GDB=$(toolchain_prefix)gdb rust-gdb $(kernel) -ex "target remote :1234"

ifeq ($(arch), aarch64)
run: $(kernel)
	@$(toolchain_prefix)objcopy $(kernel) -O binary build/kernel8.img
	@qemu-system-$(arch) -machine raspi3b -serial null -serial stdio -kernel $(kernel) -display none -d int -s
else ifeq ($(arch), riscv64)
run: $(kernel)
	@$(toolchain_prefix)objcopy $(kernel) -O binary build/kernel-$(arch).img
	@qemu-system-$(arch) -machine virt -serial stdio -kernel build/kernel-$(arch).img -display none -s -S
else
run: $(iso)
	@qemu-system-$(arch) -monitor stdio -cdrom $(iso) -s
endif

iso: $(iso)

kernel:
	@cargo build --target $(target)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.$(assembly_ext)
	@mkdir -p $(shell dirname $@)
ifeq ($(arch), x86_64)
	@nasm -Wall -felf64 $< -o $@
else
	@$(toolchain_prefix)as -g -c $< -o $@
endif

$(iso): $(kernel) $(grub_cfg)
	@$(eval TMP := $(shell mktemp -d))
	@mkdir -p $(TMP)/boot/grub
	@cp $(kernel) $(TMP)/boot/kernel.bin
	@cp $(grub_cfg) $(TMP)/boot/grub
	@$(grub_mkrescue) -o $(iso) $(TMP)
	@rm -r $(TMP)

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@$(toolchain_prefix)$(linker) --nmagic -z noexecstack --script=$(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)
