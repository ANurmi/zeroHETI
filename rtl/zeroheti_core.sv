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
    input  logic                              clk_i,
    input  logic                              rst_ni,
    input  logic                              testmode_i,
    input  logic           [            63:0] mtime_i,
    input  logic                              jtag_tck_i,
    input  logic                              jtag_tms_i,
    input  logic                              jtag_trst_ni,
    input  logic                              jtag_td_i,
    output logic                              jtag_td_o,
    input  logic           [Cfg.num_irqs-1:0] ext_irqs_i,
           OBI_BUS.Manager                    obi_mgr,
           APB.Master                         apb_mgr
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
  logic [   TsWidth-1:0] irq_level;
  logic [    IrqWidth-1:0] irq_id_claim;
  logic [             1:0] irq_priv;

  edfic_top #(
      .NrIrqs (Cfg.num_irqs),
      .TsWidth(TsWidth),
      .TsClip (32'd0)
  ) i_edfic (
      .clk_i,
      .rst_ni,
      .cfg_req_i(),
      .cfg_we_i(),
      .cfg_addr_i(),
      .cfg_wdata_i(),
      .cfg_rdata_o(),
      .irq_i(),
      .irq_valid_o(),
      .irq_ack_i(),
      .irq_dl_o(),
      .irq_id_o(),
      .irq_id_i(),
      .mtime_i
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
      .inst_bus(inst_bus),
      .data_bus(data_bus),
      .imem_bus(imem_bus),
      .dmem_bus(dmem_bus),
      .intc_bus(intc_bus),
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


  superheti #(
      .NumInterrupts  (Cfg.num_irqs),
      .DmHaltAddr     (dm::HaltAddress),
      .DmExceptionAddr(dm::ExceptionAddress),
      .NumPrios       (2 ** TsWidth)
      //,.IntcBaseAddr   (zeroheti_pkg::AddrMap.intc.base)
  ) i_superheti (
      .clk_i,
      .rst_ni,
      .hart_id_i  (Cfg.hart_id),
      .boot_addr_i(Cfg.boot_addr),

      .instr_req_o   (inst_bus.req),
      .instr_addr_o  (inst_bus.addr),
      .instr_gnt_i   (inst_bus.gnt),
      .instr_rvalid_i(inst_bus.rvalid),
      .instr_rdata_i (inst_bus.rdata),
      .instr_err_i   (inst_bus.err),

      .data_req_o   (data_bus.req),
      .data_gnt_i   (data_bus.gnt),
      .data_rvalid_i(data_bus.rvalid),
      .data_we_o    (data_bus.we),
      .data_be_o    (data_bus.be),
      .data_addr_o  (data_bus.addr),
      .data_wdata_o (data_bus.wdata),
      .data_rdata_i (data_bus.rdata),
      .data_err_i   (data_bus.err),

      .irq_heti_i (irq_heti),
      .irq_nest_i (1'b0),
      .irq_i      (core_irq),
      .irq_id_o   (irq_id_claim),
      .irq_ack_o  (irq_ack),
      .irq_level_i(irq_level),

      .debug_req_i   (debug_req),
      .debug_mode_o  (),
      .fetch_enable_i(1'b1),
      .core_sleep_o  ()

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

