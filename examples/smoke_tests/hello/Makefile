CRT0     = $(CURDIR)/crt0.S
TEST_SRC = $(CURDIR)/test.c

.PHONY: compile
compile:
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(CRT0)     -o $(BUILD_DIR)/crt0.o
	$(RVGCC) $(ARCH_FLAGS) $(CFLAGS) $(TEST_SRC) -o $(BUILD_DIR)/$(TEST).o

.PHONY: link
link:
	$(RVGCC) $(LINKER)

.PHONY: test
test: compile link

