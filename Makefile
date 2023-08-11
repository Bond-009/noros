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

ifeq ($(arch), aarch64)
	image := build/kernel8.img
else ifeq ($(arch), x86_64)
	image := build/noros-$(arch).iso
else
	image := build/kernel-$(arch).img
endif

assembly_source_files := $(wildcard src/arch/$(arch)/*.$(assembly_ext))
assembly_object_files := $(patsubst src/arch/$(arch)/%.$(assembly_ext), build/arch/$(arch)/%.o, $(assembly_source_files))

rust_os := target/$(target)/debug/libnoros.a

.PHONY: clean test gdb objdump run deploy image kernel

clean:
	@rm -rf build
	@cargo clean

test:
	@cargo test --target $(shell rustc -vV | sed -n 's/host: //p')

objdump: $(kernel)
	@$(toolchain_prefix)objdump --disassemble-all --demangle $(kernel)

gdb:
	@RUST_GDB=$(toolchain_prefix)gdb rust-gdb $(kernel) -ex "target remote :1234"

run: $(image)
ifeq ($(arch), aarch64)
	@qemu-system-$(arch) -machine raspi3b -serial null -serial stdio -kernel $(kernel) -display none -d int -s
else ifeq ($(arch), riscv64)
	@qemu-system-$(arch) -machine virt -serial stdio -kernel build/kernel-$(arch).img -display none -s
else
	@qemu-system-$(arch) -serial stdio -cdrom $(image) -s
endif

ifeq ($(arch), riscv64)
deploy: $(image)
	@xfel ddr d1
	@xfel jtag
	@xfel write 0x40000000 $(image)
	@xfel exec 0x40000000
endif

image: $(image)

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

ifeq ($(arch), x86_64)
$(image): $(kernel) $(grub_cfg)
	@$(eval TMP := $(shell mktemp -d))
	@mkdir -p $(TMP)/boot/grub
	@cp $(kernel) $(TMP)/boot/kernel.bin
	@cp $(grub_cfg) $(TMP)/boot/grub
	@$(grub_mkrescue) -o $(image) $(TMP)
	@rm -r $(TMP)
else
$(image): $(kernel)
	@$(toolchain_prefix)objcopy $(kernel) -O binary $(image)
endif

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@$(toolchain_prefix)$(linker) --nmagic -z noexecstack --no-warn-rwx-segment --script=$(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)
