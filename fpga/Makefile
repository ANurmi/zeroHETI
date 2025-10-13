export REPO_DIR        ?= $(PWD)
export FPGA_BUILD_DIR  ?= $(REPO_DIR)/build/fpga
export FPGA_TCL_DIR    ?= $(REPO_DIR)/fpga/scripts
export BOARD_FILES_DIR ?= $(REPO_DIR)/fpga/board_files

export COMMON_CELLS_DIR ?= $(shell bender path common_cells)
export APB_DIR          ?= $(shell bender path apb)
export OBI_DIR          ?= $(shell bender path obi)
export AXI_DIR          ?= $(shell bender path axi)
export RTIBEX_DIR       ?= $(shell bender path rt-ibex)
export REGIF_DIR        ?= $(shell bender path register_interface)

export SRC_FILES       ?= $(shell bender script flist -t fpga -t rtl)
export PROJECT         ?= zeroHETI
export FREQ_TARGET     ?= 25

# Project synthesis defines
export FPGA_SYN_DEFINES ?= \
	FPGA=1 \
	SYNTHESIS=1 \
	COMMON_CELLS_ASSERTS_OFF=1
 		
# Project simulation defines
export FPGA_SIM_DEFINES ?= \
 	XSIM=1 \
	SYNTHESIS=1 \
 	FPGA=1 


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


