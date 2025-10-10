export REPO_DIR       ?= $(PWD)
export FPGA_BUILD_DIR ?= $(REPO_DIR)/build/fpga
export FPGA_TCL_DIR   ?= $(REPO_DIR)/fpga/scripts
export PROJECT        ?= zeroHETI

.PHONY: init
init:
	@mkdir -p $(FPGA_BUILD_DIR)

.PHONY: clean
clean:
	@rm -fr $(FPGA_BUILD_DIR)

.PHONY: syn
syn: clean init
	@echo "FPGA Test"
	vivado -mode batch -source $(FPGA_TCL_DIR)/run.tcl -notrace \
	-log $(FPGA_BUILD_DIR)/vivado_$(PROJECT).log \
	-journal $(FPGA_BUILD_DIR)/vivado_$(PROJECT).jou


