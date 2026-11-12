module zeroheti_core
  import zeroheti_pkg::*;
#(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::CLICCfg,
    localparam int unsigned IrqWidth = $clog2(Cfg.irq.num_src),
    localparam int unsigned TsWidth = 9,
    localparam int unsigned PrioWidth = 8
) (
    input  logic                                 clk_i,
    input  logic                                 rst_ni,
    input  logic                                 testmode_i,
    input  logic           [               31:0] boot_addr_i,
    input  logic           [               63:0] mtime_i,
    input  logic                                 jtag_tck_i,
    input  logic                                 jtag_tms_i,
    input  logic                                 jtag_trst_ni,
    input  logic                                 jtag_td_i,
    input  logic                                 intc_mtime_en_i,
    output logic                                 jtag_td_o,
    input  logic           [Cfg.irq.num_src-1:0] ext_irqs_i,
           AXI_LITE.Master                       ext_mgr,
           AXI_LITE.Slave                        ext_sbr
);


  localparam int unsigned DemuxNum = 3;
  localparam int unsigned DemuxWidth = $clog2(DemuxNum);
  localparam int unsigned MuxNum = 2;

  localparam int unsigned XbarNumMgr = 3;
  localparam int unsigned XbarNumSbr = 3;

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) demux_bus[DemuxNum-1:0] ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) demux_cut ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) mux_bus[MuxNum-1:0] ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) mux_cut ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) mgr_bus[XbarNumMgr-1:0] ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) sbr_bus[XbarNumSbr-1:0] ();

  APB ic_bus ();

  axi_lite_join_intf i_join_mst (
      .in (ext_sbr),
      .out(mux_bus[1])
  );

  axi_lite_join_intf i_join_slv (
      .in (demux_bus[2]),
      .out(ext_mgr)
  );

  logic [DemuxWidth-1:0] demux_aw_sel, demux_ar_sel;

  logic irq_heti, irq_ack, irq_valid, irq_shv, irq_nest;
  logic [Cfg.irq.num_src-1:0] core_irq;
  logic [       IrqWidth-1:0] irq_id;
  logic [      PrioWidth-1:0] irq_level;
  logic [       IrqWidth-1:0] irq_id_claim;
  logic [                1:0] irq_priv;

  logic [               63:0] mtime_gated;
  assign mtime_gated = (intc_mtime_en_i) ? mtime_i : 64'h0;

  zeroheti_int_ctrl #(
      .Cfg(Cfg),
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
      .mtime_en_i (intc_mtime_en_i),
      .apb_sbr    (ic_bus)
  );

  logic debug_req;

  always_comb begin : g_core_irq
    core_irq = '0;
    if (irq_valid) begin
      core_irq[irq_id] = 1'b1;
    end
  end

  always_comb begin : demux_sel

    // Default: external
    demux_aw_sel = 2;
    demux_ar_sel = 2;

    unique case (demux_cut.aw_addr) inside
      [Cfg.am.core.dbg.base : Cfg.am.core.dbg.last - 1]: demux_aw_sel = 0;
      [Cfg.am.intc.base : Cfg.am.intc.last - 1]: demux_aw_sel = 1;
      default: ;
    endcase

    unique case (demux_cut.ar_addr) inside
      [Cfg.am.core.dbg.base : Cfg.am.core.dbg.last - 1]: demux_ar_sel = 0;
      [Cfg.am.intc.base : Cfg.am.intc.last - 1]: demux_ar_sel = 1;
      default: ;
    endcase

  end

  axi_lite_to_apb_intf #(
      .rule_t(axi_pkg::xbar_rule_32_t)
  ) i_axi_to_apb (
      .clk_i,
      .rst_ni,
      .slv       (demux_bus[1]),
      .paddr_o   (ic_bus.paddr),
      .pprot_o   (ic_bus.pprot),
      .pselx_o   (ic_bus.psel),
      .penable_o (ic_bus.penable),
      .pwrite_o  (ic_bus.pwrite),
      .pwdata_o  (ic_bus.pwdata),
      .pstrb_o   (ic_bus.pstrb),
      .pready_i  (ic_bus.pready),
      .prdata_i  (ic_bus.prdata),
      .pslverr_i (ic_bus.pslverr),
      .addr_map_i('0)
  );

  localparam axi_pkg::xbar_cfg_t XbarCfg = '{
      NoSlvPorts: XbarNumMgr,
      NoMstPorts: XbarNumSbr,
      MaxMstTrans: 32'd3,
      MaxSlvTrans: 32'd3,
      FallThrough: 1'b1,
      LatencyMode: axi_pkg::NO_LATENCY,
      AxiAddrWidth: 32,
      AxiDataWidth: 32,
      NoAddrRules: 2,
      default: '0
  };

  localparam axi_pkg::xbar_rule_32_t [XbarCfg.NoAddrRules-1:0] AddrMap = '{
      '{idx: 32'd2, start_addr: Cfg.am.core.dmem.base, end_addr: Cfg.am.core.dmem.last},
      '{idx: 32'd1, start_addr: Cfg.am.core.imem.base, end_addr: Cfg.am.core.imem.last}
  };

  axi_lite_mux_intf #(
      .AxiAddrWidth(32'd32),
      .AxiDataWidth(32'd32),
      .NoSlvPorts (MuxNum),
      .FallThrough(1'b1),
      .MaxTrans   (32'd1)
  ) i_mux (
      .clk_i,
      .rst_ni,
      .test_i(1'b0),
      .slv   (mux_bus),
      .mst   (mux_cut)
  );

  axi_lite_cut_intf #(
      .ADDR_WIDTH(32'd32),
      .DATA_WIDTH(32'd32)
  ) i_mux_cut (
      .clk_i,
      .rst_ni,
      .in (mux_cut),
      .out(mgr_bus[0])
  );

  axi_lite_xbar_intf #(
      .Cfg   (XbarCfg),
      .rule_t(axi_pkg::xbar_rule_32_t)
  ) i_xbar (
      .clk_i,
      .rst_ni,
      .addr_map_i           (AddrMap),
      .test_i               (1'b0),
      .slv_ports            (mgr_bus),
      .mst_ports            (sbr_bus),
      .en_default_mst_port_i('1),
      .default_mst_port_i   ('0)
  );

  axi_lite_cut_intf #(
      .ADDR_WIDTH(32'd32),
      .DATA_WIDTH(32'd32)
  ) i_demux_cut (
      .clk_i,
      .rst_ni,
      .in (sbr_bus[0]),
      .out(demux_cut)
  );

  axi_lite_demux_intf #(
      .AxiAddrWidth(32'd32),
      .AxiDataWidth(32'd32),
      .NoMstPorts(DemuxNum),
      .MaxTrans(32'd1)
  ) i_demux (
      .clk_i,
      .rst_ni,
      .slv_aw_select_i(demux_aw_sel),
      .slv_ar_select_i(demux_ar_sel),
      .test_i         (1'b0),
      .slv            (demux_cut),
      .mst            (demux_bus)
  );


  rt_ibex_wrapper #(
      .Cfg(Cfg),
      .PrioWidth(PrioWidth)
  ) i_rt_ibex (
      .clk_i,
      .rst_ni,
      .inst_mgr      (mgr_bus[1]),
      .data_mgr      (mgr_bus[2]),
      .boot_addr_i   (boot_addr_i),
      .irq_heti_i    (irq_heti),
      .core_irq_i    (core_irq),
      .irq_id_claim_o(irq_id_claim),
      .irq_ack_o     (irq_ack),
      .irq_level_i   (irq_level),
      .irq_shv_i     (irq_shv),
      .irq_priv_i    (irq_priv),
      .mtime_gated_i (mtime_gated),
      .debug_req_i   (debug_req)
  );

  zeroheti_dbg_wrapper #(
      .Cfg(Cfg)
  ) i_debug (
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
      .mem_sbr    (demux_bus[0]),
      .sba_mgr    (mux_bus[0])
  );

  axi_lite_sram #(
      .NumBanks(Cfg.imem.banks),
      .NumWords(Cfg.imem.bytes / 4),
      .SimInit ("random")
  ) i_imem (
      .clk_i,
      .rst_ni,
      .slv(sbr_bus[1])
  );
  axi_lite_sram #(
      .NumBanks(Cfg.dmem.banks),
      .NumWords(Cfg.dmem.bytes / 4),
      .SimInit ("random")
  ) i_dmem (
      .clk_i,
      .rst_ni,
      .slv(sbr_bus[2])
  );

endmodule : zeroheti_core

