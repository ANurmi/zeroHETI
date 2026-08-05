SW_DIR ?= examples/smoke_tests

RISCOF ?= $(shell bender path riscof_verilator)

.PHONY: ips elf vlint verilate simv riscof_build riscof_run fpga clean_build clean_ips clean_all

ips:
	bender update
	bender vendor init

elf:
	@$(MAKE) -C $(SW_DIR) elf --no-print-directory

vlint:
	$(MAKE) -C verilator lint

verilate:
	$(MAKE) -C verilator verilate

simv:
	$(MAKE) -C verilator simv

riscof_build:
	$(MAKE) -C riscof compile

riscof_run:
	$(MAKE) -C riscof riscof_run

fpga:
	@$(MAKE) -C fpga syn --no-print-directory

clean_build:
	rm -fr build

clean_ips:
	rm -fr .bender

clean_all: clean_build clean_ips
