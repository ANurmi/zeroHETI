BUILD_DIR       ?= $(realpath $(CURDIR))/../build
VERIL_DIR       ?= $(realpath $(CURDIR))/../verilator

TOP_MODULE       = zeroheti_top
VERIL_TOP        = $(VERIL_DIR)/tb/$(TOP_MODULE).cc

VERIL_BUILD_DIR ?= $(BUILD_DIR)/verilator_build

BENDER_TARGETS   = -t rtl -t tech_cells_generic_exclude_deprecated
SOURCES          = $(shell bender script flist $(BENDER_TARGETS))
VBENCH           = $(shell bender path vbench)

WAIVER          ?= ./waiver.vlt

CFLAGS           = -I$(shell bender path vbench)/src

VDEFS ?= \
  +define+VERILATOR=1 \
  +define+RVFI \
  +define+COMMON_CELLS_ASSERTS_OFF

VFLAGS           = \
  $(VDEFS) \
  --cc \
  --trace-fst \
  --trace-structs \
  --trace-params \
  --hierarchical \
  --exe $(VERIL_TOP) \
  --build \
   -CFLAGS "-I$(VBENCH)/src" \
  --Mdir $(VERIL_BUILD_DIR)/obj_dir \
   -j `nproc` \

INCLUDES         = \
  -I$(shell bender path apb)/include \
  -I$(shell bender path axi)/include \
  -I$(shell bender path common_cells)/include \
  -I$(shell bender path obi)/include \
  -I$(shell bender path register_interface)/include \
  -I$(shell bender path rt-ibex)/vendor/lowrisc_ip/dv/sv/dv_utils \
  -I$(shell bender path rt-ibex)/vendor/lowrisc_ip/ip/prim/rtl \
  -I$(shell bender path rt-ibex)/rtl

.PHONY: init
init:
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(VERIL_BUILD_DIR)

.PHONY: clean
clean:
	@rm -fr $(VERIL_BUILD_DIR)

.PHONY: lint
lint:
	verilator --lint-only --top-module $(TOP_MODULE) \
	$(INCLUDES) $(WAIVER) $(SOURCES)
	@echo "Verilator lint [OK]"

.PHONY: verilate
verilate: init
	verilator $(VFLAGS) $(INCLUDES) \
	--top-module $(TOP_MODULE) \
	$(WAIVER) $(SOURCES)

.PHONY: simv
simv:
	$(VERIL_BUILD_DIR)/obj_dir/V$(TOP_MODULE) $(TEST)

