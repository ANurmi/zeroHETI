ZH_ROOT         ?= $(realpath $(CURDIR))/..
BUILD_DIR       ?= $(ZH_ROOT)/build
SW_BUILD_DIR    ?= $(ZH_ROOT)/build/sw
VERIL_DIR       ?= $(ZH_ROOT)/verilator

XLEN             = 64
TOOLCHAIN        = riscv$(XLEN)-unknown-elf-

TOP_MODULE       = zeroheti_top
RV_TOP           = zeroheti_compliance
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

INTC             = HETIC

BASE             = E
MULT             = M
COMP             = C
ISA              = RV32$(BASE)$(MULT)$(COMP)

VDEFS ?= \
  +define+VERILATOR=1 \
  +define+MOCK_UART \
	+define+ZH_ROOT=$(ZH_ROOT) \
	+define+IMEM_BYTES=$(IMEM_BYTES) \
	+define+DMEM_BYTES=$(DMEM_BYTES) \
	+define+INTC=$(INTC) \
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
  --build \
   -CFLAGS "-I$(VBENCH)/src -DZH_ROOT=$(ZH_ROOT)" \
  --Mdir $(VERIL_BUILD_DIR)/obj_dir \
   -j `nproc` \


INCLUDES         = \
  -I$(ZH_ROOT)/rtl/include \
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
	verilator \
	--lint-only \
	--top-module $(TOP_MODULE) \
	--no-timing \
	+define+SYNTHESIS \
	$(VDEFS) $(INCLUDES) $(WAIVER) $(SOURCES)
	@echo "Verilator lint [OK]"

.PHONY: hex
hex:
	$(TOOLCHAIN)objcopy $(SW_BUILD_DIR)/$(TEST).elf -O binary $(SW_BUILD_DIR)/tmp.bin
	@xxd -ps $(SW_BUILD_DIR)/tmp.bin $(SW_BUILD_DIR)/tmp.hex
	@tr -d '\n' < $(SW_BUILD_DIR)/tmp.hex > $(SW_BUILD_DIR)/tmp_nl.hex
	@sed -r 's/(.{8})/\1\n/g' < $(SW_BUILD_DIR)/tmp_nl.hex > $(SW_BUILD_DIR)/tmp_be.hex
	@sed ':a; /^.\{8\}$$/!s/$$/0/; ta' < $(SW_BUILD_DIR)/tmp_be.hex > $(SW_BUILD_DIR)/tmp_be_pad.hex
	@sed -E 's/(..)(..)(..)(..)/\4\3\2\1/' < $(SW_BUILD_DIR)/tmp_be_pad.hex > $(SW_BUILD_DIR)/tmp_le.hex

.PHONY: trim
trim:
	@head -$(shell wc -w < $(SW_BUILD_DIR)/tmp_le.hex) $(SW_BUILD_DIR)/tmp_le.hex > $(SW_BUILD_DIR)/$(TEST).hex
	@rm $(SW_BUILD_DIR)/tmp*

.PHONY: $(TEST)
$(TEST): hex trim
	@cp $(BUILD_DIR)/sw/$(TEST).hex $(VERIL_BUILD_DIR)/verilator_stim.hex
	@head -$(IMEM_WORDS)    $(VERIL_BUILD_DIR)/verilator_stim.hex > $(VERIL_BUILD_DIR)/imem_stim.hex
	@tail -n +$(DMEM_START) $(VERIL_BUILD_DIR)/verilator_stim.hex > $(VERIL_BUILD_DIR)/dmem_stim.hex

.PHONY: verilate
verilate: init
	verilator $(VFLAGS) $(INCLUDES) \
  +define+RVFI \
  --exe $(VERIL_TOP) \
	--top-module $(TOP_MODULE) \
	$(WAIVER) $(SOURCES)

.PHONY: verilate_compliance
verilate_compliance: init
	verilator $(VFLAGS) $(INCLUDES) \
  +define+RVFI \
	--top-module $(RV_TOP) \
	-Wno-UNOPTFLAT \
	--exe $(VERIL_DIR)/tb/$(RV_TOP).cc \
	$(WAIVER) $(SOURCES) $(VERIL_DIR)/tb/$(RV_TOP).sv

.PHONY: simv
simv: $(TEST)
	$(VERIL_BUILD_DIR)/obj_dir/V$(TOP_MODULE) $(TEST) --load=$(LOAD)

