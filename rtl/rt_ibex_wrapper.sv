module rt_ibex_wrapper #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::CpssCfg,
    parameter int unsigned PrioWidth = 8,
    localparam int unsigned NumIrq = Cfg.irq.num_src,
    localparam int unsigned IrqWidth = $clog2(Cfg.irq.num_src)
) (
    input  logic                           clk_i,
    input  logic                           rst_ni,
    input  logic           [         31:0] boot_addr_i,
    input  logic                           irq_heti_i,
    output logic                           irq_ack_o,
    input  logic           [PrioWidth-1:0] irq_level_i,
    input  logic                           irq_shv_i,
    input  logic           [          1:0] irq_priv_i,
    output logic           [ IrqWidth-1:0] irq_id_o,
    output logic           [ IrqWidth-1:0] irq_id_claim_o,
    input  logic           [   NumIrq-1:0] core_irq_i,
    input  logic           [         63:0] mtime_gated_i,
    input  logic                           debug_req_i,
           AXI_LITE.Master                 inst_mgr,
           AXI_LITE.Master                 data_mgr
);

  logic inst_req;

  assign inst_mgr.ar_valid = inst_req;
  assign inst_mgr.aw_addr = 32'h0;
  assign inst_mgr.r_ready = 1'b1;

  logic [31:0] data_addr;
  logic data_req, data_gnt, data_we, data_rvalid, data_err;

  assign data_mgr.aw_addr = data_addr;
  assign data_mgr.ar_addr = data_addr;

  assign data_mgr.aw_valid = (data_req &  data_we);
  assign data_mgr.ar_valid = (data_req & !data_we);

  assign data_mgr.w_valid  = data_we;

  assign data_gnt    = data_mgr.ar_ready  | data_mgr.aw_ready;
  assign data_rvalid = data_mgr.r_valid   | data_mgr.b_valid;
  assign data_err    = data_mgr.r_resp[1] & data_mgr.r_valid
                     | data_mgr.b_resp[1] & data_mgr.b_valid;

  `ifndef SYNTHESIS
  ibex_top_tracing #(
  `else
  ibex_top #(
  `endif
      .PMPEnable       (0),
      .PMPGranularity  (0),
      .PMPNumRegions   (4),
      .MHPMCounterNum  (0),
      .MHPMCounterWidth(40),
      .RV32E           (Cfg.core.rve),
      .RV32M           (Cfg.core.mul),
      .RV32B           (ibex_pkg::RV32BNone),
      .WritebackStage  (Cfg.core.wb_stage),
      .RegFile         (Cfg.core.rf),
      .ICache          (1'b0),
      .ICacheECC       (1'b0),
      .ICacheScramble  (1'b0),
      .BranchPredictor (1'b0),
      .SecureIbex      (1'b0),
      .CLIC            (1'b1),
      .HardwareStacking(1'b0),
      .NumInterrupts   (Cfg.irq.num_src),
      .RndCnstLfsrSeed (ibex_pkg::RndCnstLfsrSeedDefault),
      .RndCnstLfsrPerm (ibex_pkg::RndCnstLfsrPermDefault),
      .DbgTriggerEn    (0),
      .DmHaltAddr      (32'(dm::HaltAddress)),
      .DmExceptionAddr (32'(dm::ExceptionAddress)),
      .MClicBaseAddr   (Cfg.am.intc.base),
      .BranchTargetALU (Cfg.core.bt_alu)
  ) i_rt_ibex (
      .clk_i,
      .rst_ni,
      .scan_rst_ni(1'b0),
      .ram_cfg_i  (10'b0),
      .hart_id_i  (Cfg.core.hart_id),
      .test_en_i  (1'b0),
      .boot_addr_i(boot_addr_i),

      .instr_req_o       (inst_req),
      .instr_addr_o      (inst_mgr.ar_addr),
      .instr_gnt_i       (inst_mgr.ar_ready),
      .instr_rvalid_i    (inst_mgr.r_valid),
      .instr_rdata_i     (inst_mgr.r_data),
      .instr_rdata_intg_i(7'b0),
      .instr_err_i       (inst_mgr.r_resp[0]),

      .data_req_o       (data_req),
      .data_gnt_i       (data_gnt),
      .data_rvalid_i    (data_rvalid),
      .data_we_o        (data_we),
      .data_be_o        (data_mgr.w_strb),
      .data_addr_o      (data_addr),
      .data_wdata_o     (data_mgr.w_data),
      .data_rdata_i     (data_mgr.r_data),
      .data_err_i       (data_err),
      .data_rdata_intg_i(7'b0),
      .data_wdata_intg_o(  /*NC*/),

      .irq_is_pcs_i(irq_heti_i),
      .irq_i       (core_irq_i),
      .irq_id_o    (irq_id_claim_o),
      .irq_ack_o   (irq_ack_o),
      .irq_level_i (8'(irq_level_i)),
      .irq_shv_i   (irq_shv_i),
      .irq_priv_i  (irq_priv_i),
      .mtime_i     (mtime_gated_i),

      .scramble_key_valid_i(1'b0),
      .scramble_key_i      (128'b0),
      .scramble_nonce_i    (64'b0),
      .scramble_req_o      (),

      .debug_req_i        (debug_req_i),
      .debug_mode_o       (),
      .crash_dump_o       (),
      .double_fault_seen_o(),
      .fetch_enable_i     (4'b0101),
      .core_sleep_o       (),

      .alert_minor_o         (),
      .alert_major_internal_o(),
      .alert_major_bus_o     ()
  );

  assign inst_mgr.aw_valid = 1'b0;
  assign inst_mgr.w_valid  = 1'b0;
  assign inst_mgr.b_ready  = 1'b0;
  assign inst_mgr.aw_prot  = '0;
  assign inst_mgr.w_data   = '0;
  assign inst_mgr.w_strb   = '0;
  assign inst_mgr.ar_prot  = '0;

  assign data_mgr.aw_prot  = '0;
  assign data_mgr.b_ready  = 1'b1;
  assign data_mgr.ar_prot  = '0;
  assign data_mgr.r_ready  = 1'b1;

endmodule : rt_ibex_wrapper
