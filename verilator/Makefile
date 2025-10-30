ZH_ROOT         ?= $(realpath $(CURDIR))/..
BUILD_DIR       ?= $(ZH_ROOT)/build
VERIL_DIR       ?= $(ZH_ROOT)/verilator

TOP_MODULE       = zeroheti_top
VERIL_TOP        = $(VERIL_DIR)/tb/$(TOP_MODULE).cc

VERIL_BUILD_DIR ?= $(BUILD_DIR)/verilator_build

BENDER_TARGETS   = -t rtl -t tech_cells_generic_exclude_deprecated
SOURCES          = $(shell bender script flist $(BENDER_TARGETS))
VBENCH           = $(shell bender path vbench)

WAIVER          ?= ./waiver.vlt

CFLAGS           = -I$(shell bender path vbench)/src

LOAD            ?= READMEM

IMEM_BYTES       = 16384
DMEM_BYTES       = 8192

# IMEM base 0x1_0000, DMEM base 0x2_0000
DMEM_REL_OFFS_WORDS = 16385

IMEM_WORDS       = $(shell echo $$(($(IMEM_BYTES) / 4)))
DMEM_START       = $(DMEM_REL_OFFS_WORDS)

ISA              = rv32ec

VDEFS ?= \
  +define+VERILATOR=1 \
  +define+MOCK_UART \
	+define+ZH_ROOT=$(ZH_ROOT) \
	+define+IMEM_BYTES=$(IMEM_BYTES) \
	+define+DMEM_BYTES=$(DMEM_BYTES) \
	+define+LOAD=$(LOAD) \
  +define+COMMON_CELLS_ASSERTS_OFF \
	+define+CORE_CFG=$(ISA)Cfg \

VFLAGS           = \
  $(VDEFS) \
  --cc \
	--timing \
  --trace-fst \
  --trace-structs \
  --trace-params \
  --hierarchical \
  --exe $(VERIL_TOP) \
  --build \
   -CFLAGS "-I$(VBENCH)/src -DZH_ROOT=$(ZH_ROOT)" \
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
	+define+SYNTHESIS \
	$(VDEFS) $(INCLUDES) $(WAIVER) $(SOURCES)
	@echo "Verilator lint [OK]"

.PHONY: $(TEST)
$(TEST):
	@cp $(BUILD_DIR)/sw/$(TEST).hex $(VERIL_BUILD_DIR)/verilator_stim.hex
	@head -$(IMEM_WORDS)    $(VERIL_BUILD_DIR)/verilator_stim.hex > $(VERIL_BUILD_DIR)/imem_stim.hex
	@tail -n +$(DMEM_START) $(VERIL_BUILD_DIR)/verilator_stim.hex > $(VERIL_BUILD_DIR)/dmem_stim.hex

.PHONY: verilate
verilate: init
	verilator $(VFLAGS) $(INCLUDES) \
  +define+RVFI \
	--top-module $(TOP_MODULE) \
	$(WAIVER) $(SOURCES)

.PHONY: simv
simv: $(TEST)
	$(VERIL_BUILD_DIR)/obj_dir/V$(TOP_MODULE) $(TEST) --load=$(LOAD)

