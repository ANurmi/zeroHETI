.PHONY: clone
clone:
	@if [ -d "./riscv-arch-test" ]; then \
		echo "riscv-arch-test already cloned!"; \
	else \
		riscof arch-test --clone; \
	fi

.PHONY: rv32e
rv32e: clone
	riscof run --suite ./riscv-arch-test/riscv-test-suite/rv32i_m/I \
	--env ./riscv-arch-test/riscv-test-suite/env --config=./config.ini

.PHONY: riscof_all
riscof_all: rv32e
