export REPO_DIR                 ?= $(PWD)
export FPGA_BUILD_DIR           ?= $(REPO_DIR)/build/fpga
export FPGA_TCL_DIR             ?= $(REPO_DIR)/fpga/scripts
export FPGA_IP_DIR              ?= $(REPO_DIR)/fpga/ips
export FPGA_IP_BUILD_DIR        ?= $(FPGA_BUILD_DIR)/ips
export FPGA_BOARD_CONFIG_SCRIPT ?= $(REPO_DIR)/fpga/scripts/pynq-z1.tcl
export FPGA_CONSTR_DIR          ?= $(REPO_DIR)/fpga/constraints
export FPGA_BOARD               ?= PYNQ-Z1

export BOARD_FILES_DIR ?= $(REPO_DIR)/fpga/board_files

export COMMON_CELLS_DIR ?= $(shell bender path common_cells)
export APB_DIR          ?= $(shell bender path apb)
export OBI_DIR          ?= $(shell bender path obi)
export AXI_DIR          ?= $(shell bender path axi)
export RTIBEX_DIR       ?= $(shell bender path rt-ibex)
export REGIF_DIR        ?= $(shell bender path register_interface)

export SRC_FILES       ?= $(shell bender script flist -t fpga -t rtl)
export PROJECT_NAME    ?= zeroHETI
export FREQ_TARGET     ?= 25

# List of FPGA IPs used in design !!MAKE SURE THESE ARE SEPARATED BY EXACTLY ONE SPACE!!
export FPGA_IP_LIST ?= \
	top_clock


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

.PHONY: all_ips
all_ips: init $(FPGA_IP_LIST)
	@echo "running all_ips recipe"

# call corresponding Makefile for each IP in FPGA_IP_LIST
$(FPGA_IP_LIST):
	$(MAKE) -C $(FPGA_IP_DIR)/$@ build_ip IP_PROJECT=$@

.PHONY: clean
clean:
	@rm -fr $(FPGA_BUILD_DIR)

.PHONY: syn
syn: clean init all_ips
	vivado -mode batch -source $(FPGA_TCL_DIR)/run.tcl -notrace \
	-log $(FPGA_BUILD_DIR)/vivado_$(PROJECT_NAME).log \
	-journal $(FPGA_BUILD_DIR)/vivado_$(PROJECT_NAME).jou


