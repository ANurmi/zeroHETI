RISCOF_PATH = $(shell bender path riscof_verilator)

export DUT_WRAPPER = $(realpath .)/dut_rt_ibex.sv
export DUT_SRC     = $(shell bender script flist-plus -p rt-ibex)

export DUT_INCLUDES    ?= \
	-I$(shell bender path rt-ibex)/vendor/lowrisc_ip/dv/sv/dv_utils \
  -I$(shell bender path rt-ibex)/vendor/lowrisc_ip/ip/prim/rtl \
  -I$(shell bender path rt-ibex)/rtl \

.PHONY: compile riscof_clone riscof_run install_sail

compile:
	mkdir -p $(RISCOF_PATH)/build
	$(MAKE) -C $(RISCOF_PATH) compile

install_sail:
	$(MAKE) -C $(RISCOF_PATH) install_sail

riscof_clone:
	$(MAKE) -C $(RISCOF_PATH) riscof_clone

riscof_run: riscof_clone
	$(MAKE) -C $(RISCOF_PATH) riscof_run
