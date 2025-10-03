CRT0     = $(CURDIR)/crt0.S
TEST_SRC = $(CURDIR)/test.c
CFLAGS   = -O0 -ffunction-sections -fdata-sections -g -c

.PHONY: compile
compile:
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(INCLUDE_DIR) $(CRT0)     -o $(BUILD_DIR)/crt0.o
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(INCLUDE_DIR) $(TEST_SRC) -o $(BUILD_DIR)/$(TEST).o

.PHONY: link
link:
	$(RVGCC) \
	$(ARCH_FLAGS) \
	-T$(LINKER) \
	-nostartfiles \
	$(INCLUDE_DIR) \
	-o $(BUILD_DIR)/$(TEST).elf \
	$(BUILD_DIR)/crt0.o \
	$(BUILD_DIR)/$(TEST).o
	#Clean up
	rm $(BUILD_DIR)/crt0.o $(BUILD_DIR)/$(TEST).o


.PHONY: test
test: compile link

