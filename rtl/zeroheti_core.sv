module zeroheti_core
  import zeroheti_pkg::*;
#(
    parameter zeroheti_pkg::core_cfg_t Cfg = zeroheti_pkg::DefaultCfg,
    localparam int unsigned IrqWidth = $clog2(Cfg.num_irqs),
    localparam int unsigned TsWidth = 14,
    localparam int unsigned PrioWidth = 8
    //localparam int unsigned PrioWidth = (zeroheti_pkg::IntController == EDFIC) ? TsWidth : $clog2(
    //    Cfg.num_prio
    //)
) (
    input  logic                         clk_i,
    input  logic                         rst_ni,
    input  logic                         testmode_i,
    input  logic      [            63:0] mtime_i,
    input  logic                         jtag_tck_i,
    input  logic                         jtag_tms_i,
    input  logic                         jtag_trst_ni,
    input  logic                         jtag_td_i,
    output logic                         jtag_td_o,
    input  logic      [Cfg.num_irqs-1:0] ext_irqs_i,
       OBI_BUS.Manager                   obi_mgr,
           APB.Master                    apb_mgr
);

  OBI_BUS inst_bus ();
  OBI_BUS data_bus ();
  OBI_BUS imem_bus ();
  OBI_BUS dmem_bus ();
  OBI_BUS intc_bus ();
  OBI_BUS per_bus ();
  OBI_BUS sba_bus ();
  OBI_BUS dbg_bus ();

  logic irq_heti, irq_ack, irq_valid, irq_shv, irq_nest;
  logic [Cfg.num_irqs-1:0] core_irq;
  logic [    IrqWidth-1:0] irq_id;
  logic [   PrioWidth-1:0] irq_level;
  logic [    IrqWidth-1:0] irq_id_claim;
  logic [             1:0] irq_priv;

  zeroheti_int_ctrl #(
.CoreCfg(Cfg),
      .TsWidth(TsWidth)
  ) i_int_ctrl (
      .clk_i,
      .rst_ni,
      .ext_irqs_i,
      .mtime_i,
      .irq_heti_o (irq_heti),
      .irq_nest_o (irq_nest),
      .irq_id_o   (irq_id),
      .irq_valid_o(irq_valid),
      .irq_id_i   (irq_id_claim),
      .irq_ack_i  (irq_ack),
      .irq_level_o(irq_level),
      .irq_priv_o (irq_priv),
      .irq_shv_o  (irq_shv),
      .obi_sbr    (intc_bus)
  );

  obi_to_apb_intf i_obi_to_apb (
      .clk_i,
      .rst_ni,
      .obi_i(per_bus),
      .apb_o(apb_mgr)
  );

  zeroheti_xbar i_xbar (
      .clk_i,
      .rst_ni,
      .inst_bus (inst_bus),
      .data_bus (data_bus),
      .imem_bus (imem_bus),
      .dmem_bus (dmem_bus),
      .intc_bus (intc_bus),
      .per_bus (per_bus),
      .sba_bus (sba_bus),
      .dbg_bus (dbg_bus),
      .mbx_bus (obi_mgr)
    );

  logic debug_req;

  always_comb begin : g_core_irq
    core_irq = '0;
    if (irq_valid) begin
      core_irq[irq_id] = 1'b1;
    end
  end


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
      .RV32E           (Cfg.rve),
      .RV32M           (Cfg.mul),
      .RV32B           (ibex_pkg::RV32BNone),
      .WritebackStage  (Cfg.wb_stage),
      .RegFile         (ibex_pkg::RegFileFF),
      .ICache          (1'b0),
      .ICacheECC       (1'b0),
      .ICacheScramble  (1'b0),
      .BranchPredictor (1'b0),
      .SecureIbex      (1'b0),
      .CLIC            (1'b1),
      .HardwareStacking(1'b0),
      .NumInterrupts   (Cfg.num_irqs),
      .RndCnstLfsrSeed (ibex_pkg::RndCnstLfsrSeedDefault),
      .RndCnstLfsrPerm (ibex_pkg::RndCnstLfsrPermDefault),
      .DbgTriggerEn    (0),
      .DmHaltAddr      (dm::HaltAddress),
      .DmExceptionAddr (dm::ExceptionAddress),
      .MClicBaseAddr   (zeroheti_pkg::AddrMap.intc.base),
      .BranchTargetALU (Cfg.bt_alu)
  ) i_rt_ibex (
      .clk_i,
      .rst_ni,
      .scan_rst_ni(1'b0),
      .ram_cfg_i  (10'b0),
      .hart_id_i  (Cfg.hart_id),
      .test_en_i  (testmode_i),
      .boot_addr_i(Cfg.boot_addr),

      .instr_req_o       (inst_bus.req),
      .instr_addr_o      (inst_bus.addr),
      .instr_gnt_i       (inst_bus.gnt),
      .instr_rvalid_i    (inst_bus.rvalid),
      .instr_rdata_i     (inst_bus.rdata),
      .instr_rdata_intg_i(7'b0),
      .instr_err_i       (inst_bus.err),

      .data_req_o       (data_bus.req),
      .data_gnt_i       (data_bus.gnt),
      .data_rvalid_i    (data_bus.rvalid),
      .data_we_o        (data_bus.we),
      .data_be_o        (data_bus.be),
      .data_addr_o      (data_bus.addr),
      .data_wdata_o     (data_bus.wdata),
      .data_rdata_i     (data_bus.rdata),
      .data_err_i       (data_bus.err),
      .data_rdata_intg_i(7'b0),
      .data_wdata_intg_o(),

      .irq_is_pcs_i(irq_heti),
      .irq_i       (core_irq),
      .irq_id_o    (irq_id_claim),
      .irq_ack_o   (irq_ack),
      .irq_level_i (8'(irq_level)),
      .irq_shv_i   (irq_shv),
      .irq_priv_i  (irq_priv),

      .scramble_key_valid_i(1'b0),
      .scramble_key_i      (128'b0),
      .scramble_nonce_i    (64'b0),
      .scramble_req_o      (),

      .debug_req_i        (debug_req),
      .debug_mode_o       (),
      .crash_dump_o       (),
      .double_fault_seen_o(),
      .fetch_enable_i     (4'b0101),
      .core_sleep_o       (),

      .alert_minor_o         (),
      .alert_major_internal_o(),
      .alert_major_bus_o     ()
  );

  // CPU tie-offs
  assign inst_bus.reqpar = 1'b0;
  assign inst_bus.aid    = 1'b0;
  assign inst_bus.a_optional = 1'b0;
  assign inst_bus.be    = 4'b0;
  assign inst_bus.we    = 1'b0;
  assign inst_bus.wdata = 32'b0;
  
  assign data_bus.aid    = 1'b0;
  assign data_bus.a_optional = 1'b0;

/*
  assign mgr_bus[2].reqpar = 1'b0;
  assign mgr_bus[2].aid    = 1'b0;
  assign mgr_bus[2].a_optional = 1'b0;
*/
  zeroheti_dbg_wrapper #() i_debug (
      .clk_i,
      .rst_ni,
      .testmode_i,
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .ndmreset_o (),
      .debug_req_o(debug_req),
      .mem_sbr    (dbg_bus),
      .sba_mgr    (sba_bus)
  );

  obi_mb_sram_intf #(
      .NrBanks (32'd4),
      .BaseAddr(AddrMap.imem.base),
      .NumWords(zeroheti_pkg::ImemWSize)
  ) i_imem (
      .clk_i,
      .rst_ni,
      .sbr(imem_bus)
  );

  obi_mb_sram_intf #(
      .NrBanks (32'd2),
      .BaseAddr(AddrMap.dmem.base),
      .NumWords(zeroheti_pkg::DmemWSize)
  ) i_dmem (
      .clk_i,
      .rst_ni,
      .sbr(dmem_bus)
  );

endmodule : zeroheti_core

