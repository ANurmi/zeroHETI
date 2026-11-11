SW_DIR ?= examples/smoke_tests

RISCOF ?= $(shell bender path riscof_verilator)

DUT = zeroheti

.PHONY: ips elf vlint verilate simv riscof_build riscof_run fpga clean_build clean_ips clean_all addr_map_check addr_map_gen

ips:
	bender checkout

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

addr_map_check:
	uv run mmap/scripts/check_syntax.py     mmap/src/local.yml
	uv run mmap/scripts/check_memory_map.py mmap/src/local.yml

addr_map_gen:
	uv run mmap/scripts/gen_address_map_pkg.py mmap/src/local.yml \
	-o rtl/gen/$(DUT)_address_map_pkg.sv --prefix $(DUT)

fpga:
	@$(MAKE) -C fpga syn --no-print-directory

clean_build:
	rm -fr build

clean_ips:
	rm -fr .bender

clean_all: clean_build clean_ips
