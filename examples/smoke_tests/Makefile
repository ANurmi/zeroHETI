export BUILD_DIR   =$(CURDIR)/../../build/sw
export XLEN        =32
export TOOLCHAIN   =riscv$(XLEN)-unknown-elf-
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
	xxd -ps $(BUILD_DIR)/tmp.bin $(BUILD_DIR)/tmp.hex
	tr -d '\n' < $(BUILD_DIR)/tmp.hex > $(BUILD_DIR)/tmp_nl.hex
	sed -r 's/(.{8})/\1\n/g' < $(BUILD_DIR)/tmp_nl.hex > $(BUILD_DIR)/tmp_be.hex
	sed -E 's/(..)(..)(..)(..)/\4\3\2\1/' < $(BUILD_DIR)/tmp_be.hex > $(BUILD_DIR)/tmp_le.hex

.PHONY: trim
trim:
	head -$(shell wc -w < $(BUILD_DIR)/tmp_le.hex) $(BUILD_DIR)/tmp_le.hex > $(BUILD_DIR)/$(TEST).hex
	rm $(BUILD_DIR)/tmp*

.PHONY: test
test:
	@echo "TEST not set!"
	exit 1
