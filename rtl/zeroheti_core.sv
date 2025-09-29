module zeroheti_core import zeroheti_pkg::*; #(
  core_cfg_t Cfg = DefaultCfg
)(
  input  logic clk_i,
  input  logic rst_ni,
  input  logic testmode_i,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o
);

OBI_BUS if_bus (), lsu_bus (), sba_bus ();
OBI_BUS dbg_mem_bus (), inst_bus (), data_bus (), apb_bus ();

zeroheti_core_xbar #(
) i_xbar (
  .clk_i,
  .rst_ni,
  .cpu_if   (if_bus),
  .cpu_lsu  (lsu_bus),
  .sba_mgr  (sba_bus),
  .dbg_sbr  (dbg_mem_bus),
  .inst_sbr (inst_bus),
  .data_sbr (data_bus),
  .apb_sbr  (apb_bus)
);

`ifndef SYNTHESIS
ibex_top_tracing #(
`else
ibex_top #(
`endif
  .PMPEnable        (0),
  .PMPGranularity   (0),
  .PMPNumRegions    (4),
  .MHPMCounterNum   (0),
  .MHPMCounterWidth (40),
  .RV32E            (Cfg.rve),
  .RV32M            (ibex_pkg::RV32MFast),
  .RV32B            (ibex_pkg::RV32BNone),
  .WritebackStage   (Cfg.wb_stage),
  .RegFile          (ibex_pkg::RegFileWindowFF),
  .ICache           (1'b0),
  .ICacheECC        (1'b0),
  .ICacheScramble   (1'b0),
  .BranchPredictor  (1'b0),
  .SecureIbex       (1'b0),
  .CLIC             (1'b1),
  .HardwareStacking (1'b0),
  .NumInterrupts    (Cfg.num_irqs),
  .RndCnstLfsrSeed  (ibex_pkg::RndCnstLfsrSeedDefault),
  .RndCnstLfsrPerm  (ibex_pkg::RndCnstLfsrPermDefault),
  .DbgTriggerEn     (0),
  .DmHaltAddr       (dm::HaltAddress),
  .DmExceptionAddr  (dm::ExceptionAddress),
  .MClicBaseAddr    (zeroheti_pkg::AddrMap.clic.base),
  .BranchTargetALU  (Cfg.bt_alu)
) i_rt_ibex (
  .clk_i,
  .rst_ni,
  .scan_rst_ni            (1'b0),
  .ram_cfg_i              (10'b0),
  .hart_id_i              (Cfg.hart_id),
  .test_en_i              (testmode_i),
  .boot_addr_i            (Cfg.boot_addr),

  .instr_req_o            (if_bus.req),
  .instr_addr_o           (if_bus.addr),
  .instr_gnt_i            (if_bus.gnt),
  .instr_rvalid_i         (if_bus.rvalid),
  .instr_rdata_i          (if_bus.rdata),
  .instr_rdata_intg_i     (7'b0),
  .instr_err_i            (if_bus.err),

  .data_req_o             (),
  .data_gnt_i             (),
  .data_rvalid_i          (),
  .data_we_o              (),
  .data_be_o              (),
  .data_addr_o            (),
  .data_wdata_o           (),
  .data_rdata_i           (),
  .data_err_i             (),
  .data_rdata_intg_i      (),
  .data_wdata_intg_o      (),

  .irq_is_pcs_i           (),
  .irq_i                  (),
  .irq_id_o               (),
  .irq_ack_o              (),
  .irq_level_i            (),
  .irq_shv_i              (),
  .irq_priv_i             (),

  .scramble_key_valid_i   (1'b0),
  .scramble_key_i         (128'b0),
  .scramble_nonce_i       (64'b0),
  .scramble_req_o         (),

  .debug_req_i            (),
  .debug_mode_o           (),
  .crash_dump_o           (),
  .double_fault_seen_o    (),
  .fetch_enable_i         (4'b0101),
  .core_sleep_o           (),

  .alert_minor_o          (),
  .alert_major_internal_o (),
  .alert_major_bus_o      ()
);

zeroheti_dbg_wrapper #(

) i_debug (
  .clk_i,
  .rst_ni,
  .testmode_i,
  .jtag_tck_i,
  .jtag_tms_i,
  .jtag_trst_ni,
  .jtag_td_i,
  .jtag_td_o
);

obi_sram_intf #() i_imem (
  .clk_i,
  .rst_ni
);

endmodule : zeroheti_core

