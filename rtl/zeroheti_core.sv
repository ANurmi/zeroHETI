module zeroheti_core
  import zeroheti_pkg::AddrMap;
#(
    parameter  zeroheti_pkg::core_cfg_t Cfg       = zeroheti_pkg::DefaultCfg,
    localparam int unsigned             IrqWidth  = $clog2(Cfg.num_irqs),
    localparam int unsigned             PrioWidth = $clog2(Cfg.num_prio)
) (
    input  logic                         clk_i,
    input  logic                         rst_ni,
    input  logic                         testmode_i,
    input  logic                         jtag_tck_i,
    input  logic                         jtag_tms_i,
    input  logic                         jtag_trst_ni,
    input  logic                         jtag_td_i,
    output logic                         jtag_td_o,
    input  logic      [Cfg.num_irqs-1:0] ext_irqs_i,
           APB.Master                    apb_mgr
);

  localparam int unsigned NumSbrPorts = 32'd3;
  localparam int unsigned NumMgrPorts = 32'd6;
  localparam int unsigned NumAddrRules = NumMgrPorts;  // works for single, continuous regions

  localparam bit [NumSbrPorts-1:0][NumMgrPorts-1:0] Connectivity = '{
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1},
      '{1'b0, 1'b0, 1'b0, 1'b0, 1'b1, 1'b1},
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1}
  };

  OBI_BUS mgr_bus[NumSbrPorts] ();
  OBI_BUS sbr_bus[NumMgrPorts] ();

  logic irq_heti, irq_ack, irq_valid;
  logic [Cfg.num_irqs-1:0] core_irq;
  logic [    IrqWidth-1:0] irq_id;
  logic [   PrioWidth-1:0] irq_level;
  logic [    IrqWidth-1:0] irq_id_claim;

  obi_hetic #(
      .NrIrqLines(Cfg.num_irqs),
      .NrIrqPrios(Cfg.num_prio)
  ) i_hetic (
      .clk_i,
      .rst_ni,
      .ext_irqs_i,
      .irq_heti_o (irq_heti),
      .irq_nest_o (  /*not used yet*/),
      .irq_id_o   (irq_id),
      .irq_valid_o(irq_valid),
      .irq_id_i   (irq_id_claim),
      .irq_ack_i  (irq_ack),
      .irq_level_o(irq_level),
      .obi_sbr    (sbr_bus[3])
  );

  obi_to_apb_intf i_obi_to_apb (
      .clk_i,
      .rst_ni,
      .obi_i(sbr_bus[4]),
      .apb_o(apb_mgr)
  );

  typedef struct packed {
    int unsigned idx;
    logic [31:0] start_addr;
    logic [31:0] end_addr;
  } addr_map_rule_t;

  localparam addr_map_rule_t [NumAddrRules-1:0] CoreAddrMap = '{
      '{idx: 0, start_addr: AddrMap.dbg.base, end_addr: AddrMap.dbg.last},
      '{idx: 1, start_addr: AddrMap.imem.base, end_addr: AddrMap.imem.last},
      '{idx: 2, start_addr: AddrMap.dmem.base, end_addr: AddrMap.dmem.last},
      '{idx: 3, start_addr: AddrMap.hetic.base, end_addr: AddrMap.hetic.last},
      // TODO: map other APB peripherals
      '{
          idx: 4,
          start_addr: AddrMap.uart.base,
          end_addr: AddrMap.i2c.last
      },
      '{idx: 5, start_addr: AddrMap.ext.base, end_addr: AddrMap.ext.last}
  };

  // TODO: add ext port when needed
  assign sbr_bus[5].gnt       = 1'b0;
  assign sbr_bus[5].gntpar    = 1'b0;
  assign sbr_bus[5].err       = 1'b0;
  assign sbr_bus[5].rready    = 1'b0;
  assign sbr_bus[5].rreadypar = 1'b0;
  assign sbr_bus[5].rvalid    = 1'b0;
  assign sbr_bus[5].rdata     = 32'b0;
  assign sbr_bus[5].rvalidpar = 1'b0;
  assign sbr_bus[5].rid       = 1'b0;
  assign sbr_bus[5].r_optional= 1'b0;

  logic debug_req;

  always_comb begin : g_core_irq
    core_irq = '0;
    if (irq_valid) begin
      core_irq[irq_id] = 1'b1;
    end
  end

  obi_xbar_intf #(
      .NumSbrPorts    (NumSbrPorts),
      .NumMgrPorts    (NumMgrPorts),
      .NumMaxTrans    (32'd1),
      .NumAddrRules   (NumAddrRules),
      .addr_map_rule_t(addr_map_rule_t),
      .Connectivity   (Connectivity)
  ) i_xbar (
      .clk_i,
      .rst_ni,
      .testmode_i,
      .addr_map_i      (CoreAddrMap),
      .en_default_idx_i(3'b0),
      .default_idx_i   (9'b0),
      .sbr_ports       (mgr_bus),
      .mgr_ports       (sbr_bus)
  );


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
      .MClicBaseAddr   (zeroheti_pkg::AddrMap.hetic.base),
      .BranchTargetALU (Cfg.bt_alu)
  ) i_rt_ibex (
      .clk_i,
      .rst_ni,
      .scan_rst_ni(1'b0),
      .ram_cfg_i  (10'b0),
      .hart_id_i  (Cfg.hart_id),
      .test_en_i  (testmode_i),
      .boot_addr_i(Cfg.boot_addr),

      .instr_req_o       (mgr_bus[1].req),
      .instr_addr_o      (mgr_bus[1].addr),
      .instr_gnt_i       (mgr_bus[1].gnt),
      .instr_rvalid_i    (mgr_bus[1].rvalid),
      .instr_rdata_i     (mgr_bus[1].rdata),
      .instr_rdata_intg_i(7'b0),
      .instr_err_i       (mgr_bus[1].err),

      .data_req_o       (mgr_bus[2].req),
      .data_gnt_i       (mgr_bus[2].gnt),
      .data_rvalid_i    (mgr_bus[2].rvalid),
      .data_we_o        (mgr_bus[2].we),
      .data_be_o        (mgr_bus[2].be),
      .data_addr_o      (mgr_bus[2].addr),
      .data_wdata_o     (mgr_bus[2].wdata),
      .data_rdata_i     (mgr_bus[2].rdata),
      .data_err_i       (mgr_bus[2].err),
      .data_rdata_intg_i(7'b0),
      .data_wdata_intg_o(),

      .irq_is_pcs_i(irq_heti),
      .irq_i       (core_irq),
      .irq_id_o    (irq_id_claim),
      .irq_ack_o   (irq_ack),
      .irq_level_i (8'(irq_level)),
      .irq_shv_i   (1'b1),
      .irq_priv_i  (2'b11),

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
  assign mgr_bus[1].reqpar = 1'b0;
  assign mgr_bus[1].aid    = 1'b0;
  assign mgr_bus[1].a_optional = 1'b0;
  assign mgr_bus[1].be    = 4'b0;
  assign mgr_bus[1].we    = 1'b0;
  assign mgr_bus[1].wdata = 32'b0;
  
  assign mgr_bus[2].reqpar = 1'b0;
  assign mgr_bus[2].aid    = 1'b0;
  assign mgr_bus[2].a_optional = 1'b0;

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
      .mem_sbr    (sbr_bus[0]),
      .sba_mgr    (mgr_bus[0])
  );

  obi_sram_intf #(
      .BaseAddr(AddrMap.imem.base),
      .NumWords(zeroheti_pkg::ImemWSize)
  ) i_imem (
      .clk_i,
      .rst_ni,
      .sbr(sbr_bus[1])
  );

  obi_sram_intf #(
      .BaseAddr(AddrMap.dmem.base),
      .NumWords(zeroheti_pkg::DmemWSize)
  ) i_dmem (
      .clk_i,
      .rst_ni,
      .sbr(sbr_bus[2])
  );

endmodule : zeroheti_core

