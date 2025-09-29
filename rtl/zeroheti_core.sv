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
  .scan_rst_ni            (),
  .ram_cfg_i              (),
  .hart_id_i              (),
  .test_en_i              (testmode_i),
  .boot_addr_i            (),

  .instr_req_o            (),
  .instr_addr_o           (),
  .instr_gnt_i            (),
  .instr_rvalid_i         (),
  .instr_rdata_i          (),
  .instr_rdata_intg_i     (),
  .instr_err_i            (),

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

  .scramble_key_valid_i   (),
  .scramble_key_i         (),
  .scramble_nonce_i       (),
  .scramble_req_o         (),

  .debug_req_i            (),
  .debug_mode_o           (),
  .crash_dump_o           (),
  .double_fault_seen_o    (),
  .fetch_enable_i         (),
  .core_sleep_o           (),

  .alert_minor_o          (),
  .alert_major_internal_o (),
  .alert_major_bus_o      ()
);

zeroheti_dbg_wrapper #() i_debug (
  .clk_i,
  .rst_ni,
  .testmode_i,
  .jtag_tck_i,
  .jtag_tms_i,
  .jtag_trst_ni,
  .jtag_td_i,
  .jtag_td_o
);

endmodule : zeroheti_core

