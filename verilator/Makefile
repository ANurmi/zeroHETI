BUILD_DIR       ?= $(realpath $(CURDIR))/../build
VERIL_DIR       ?= $(realpath $(CURDIR))/../verilator

TOP_MODULE       = zeroheti_top
VERIL_TOP        = $(VERIL_DIR)/tb/$(TOP_MODULE).cc

VERIL_BUILD_DIR ?= $(BUILD_DIR)/verilator_build

BENDER_TARGETS   = -t rtl -t tech_cells_generic_exclude_deprecated
SOURCES          = $(shell bender script flist $(BENDER_TARGETS))

CFLAGS           = -I$(shell bender path vbench)/src
INCLUDES         = -I$(shell bender path common_cells)/include \
                   -I$(shell bender path obi)/include \
                   -I$(shell bender path common_cells)/include

.PHONY: init
init:
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(VERIL_BUILD_DIR)

.PHONY: lint
lint:
	verilator --lint-only --top-module $(TOP_MODULE) \
	$(SOURCES)
	@echo "Verilator lint [OK]"
