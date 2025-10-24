.PHONY: clone
clone:
	@if [ -d "./riscv-arch-test" ]; then \
		echo "riscv-arch-test already cloned!"; \
	else \
		riscof arch-test --clone; \
	fi

.PHONY: rv32e
rv32e: clone
	@echo "Hello"
	riscof run --help

.PHONY: riscof_all
riscof_all: rv32e
