export REPO_DIR        ?= $(PWD)
export FPGA_BUILD_DIR  ?= $(REPO_DIR)/build/fpga
export FPGA_TCL_DIR    ?= $(REPO_DIR)/fpga/scripts
export BOARD_FILES_DIR ?= $(REPO_DIR)/fpga/board_files
export PROJECT         ?= zeroHETI
export FREQ_TARGET     ?= 25

.PHONY: init
init:
	@mkdir -p $(FPGA_BUILD_DIR)

.PHONY: clean
clean:
	@rm -fr $(FPGA_BUILD_DIR)

.PHONY: syn
syn: clean init
	vivado -mode batch -source $(FPGA_TCL_DIR)/run.tcl -notrace \
	-log $(FPGA_BUILD_DIR)/vivado_$(PROJECT).log \
	-journal $(FPGA_BUILD_DIR)/vivado_$(PROJECT).jou


