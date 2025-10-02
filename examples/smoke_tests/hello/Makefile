CRT0     = $(CURDIR)/crt0.S
TEST_SRC = $(CURDIR)/test.c

.PHONY: compile
compile:
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(INCLUDE_DIR) $(CRT0)     -o $(BUILD_DIR)/crt0.o
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(INCLUDE_DIR) $(TEST_SRC) -o $(BUILD_DIR)/$(TEST).o

.PHONY: link
link:
	$(RVGCC) \
	-T$(LINKER) \
	-nostartfiles \
	$(INCLUDE_DIR) \
	-o $(BUILD_DIR)/$(TEST).elf \
	$(BUILD_DIR)/crt0.o \
	$(BUILD_DIR)/$(TEST).o
	#Clean up
	rm $(BUILD_DIR)/crt0.o $(BUILD_DIR)/$(TEST).o

.PHONY: objdump
objdump:
	$(TOOLCHAIN)objdump $(BUILD_DIR)/$(TEST).elf -d > $(BUILD_DIR)/$(TEST).asm

.PHONY: test
test: compile link objdump

