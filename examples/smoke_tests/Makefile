export BUILD_DIR=$(CURDIR)/../../build/sw
export TOOLCHAIN=riscv32-unknown-elf-
export RVGCC    =$(TOOLCHAIN)gcc
# Global defaults, redefine on test-spefici level if needed
export CRT0     =$(CURDIR)/common/crt0.S
export LINKER   =$(CURDIR)/common/link.ld
export ARCH_FLAGS = -march=rv32ec_zicsr -mabi=ilp32e
export CFLAGS     = -O2 -ffunction-sections -fdata-sections -g -c

.PHONY: init
init:
	mkdir -p $(BUILD_DIR)

.PHONY: elf
elf: init
	$(MAKE) -C $(CURDIR)/$(TEST) test

.PHONY: test
test:
	@echo "TEST not set!"
