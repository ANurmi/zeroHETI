export BUILD_DIR   =$(CURDIR)/../../build/sw
export TOOLCHAIN   =riscv32-unknown-elf-
export RVGCC       =$(TOOLCHAIN)gcc
# Global defaults, redefine on test-spefic level if needed
export CRT0        =$(CURDIR)/common/crt0.S
export INCLUDE_DIR =-I$(CURDIR)/common
export LINKER      =$(CURDIR)/common/link.ld
export ARCH_FLAGS  = -march=rv32ec_zicsr -mabi=ilp32e
export CFLAGS      = -O2 -ffunction-sections -fdata-sections -g -c

.PHONY: init
init:
	mkdir -p $(BUILD_DIR)

.PHONY: elf
elf: init
	$(MAKE) -C $(CURDIR)/$(TEST) test
	$(MAKE) -C $(CURDIR) objdump
	$(MAKE) -C $(CURDIR) hex
	$(MAKE) -C $(CURDIR) trim

.PHONY: objdump
objdump:
	$(TOOLCHAIN)objdump $(BUILD_DIR)/$(TEST).elf -d > $(BUILD_DIR)/$(TEST).asm

.PHONY: hex
hex:
	$(TOOLCHAIN)objcopy $(BUILD_DIR)/$(TEST).elf -O binary $(BUILD_DIR)/tmp.bin
	xxd -R never -e $(BUILD_DIR)/tmp.bin $(BUILD_DIR)/tmp.hex
	cut -c 11-45 $(BUILD_DIR)/tmp.hex > $(BUILD_DIR)/tmp_pruned.hex
	sed -E 's/ /\n/g' $(BUILD_DIR)/tmp_pruned.hex > $(BUILD_DIR)/tmp_align.hex

.PHONY: trim
trim:
	head -$(shell wc -w < $(BUILD_DIR)/tmp_align.hex) $(BUILD_DIR)/tmp_align.hex > $(BUILD_DIR)/$(TEST).hex
	rm $(BUILD_DIR)/tmp*

.PHONY: test
test:
	@echo "TEST not set!"
	exit 1
