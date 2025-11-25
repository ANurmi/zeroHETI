SW_DIR ?= examples/smoke_tests

.PHONY: ips
ips:
	bender update

.PHONY: elf
elf:
	@$(MAKE) -C $(SW_DIR) elf --no-print-directory

.PHONY: vlint
vlint:
	$(MAKE) -C verilator lint

.PHONY: verilate
verilate:
	$(MAKE) -C verilator verilate

.PHONY: verilate_compliance
verilate_compliance:
	$(MAKE) -C verilator verilate_compliance

.PHONY: simv
simv:
	$(MAKE) -C verilator simv

.PHONY: fpga
fpga:
	@$(MAKE) -C fpga syn --no-print-directory

.PHONY: compliance
compliance:
	@$(MAKE) -C dv riscof_all --no-print-directory

.PHONY: clean_build
clean_build:
	rm -fr build

.PHONY: clean_ips
clean_ips:
	rm -fr .bender

clean_all: clean_build clean_ips
