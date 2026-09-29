`include "obi/assign.svh"

`define OBI_ASSIGN_CUT(bus)      \
  OBI_BUS bus``_cut ();          \
  obi_connection #(              \
      .Cut(1'b1)                 \
  ) bus``_cutter (               \
      .clk_i,                    \
      .rst_ni,                   \
      .obi_s(bus``),             \
      .obi_m(bus``_cut)          \
  );                             \


module zeroheti_xbar
  import zeroheti_pkg::AddrMap;
#(
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Subordinate inst_bus,
          OBI_BUS.Subordinate data_bus,
          OBI_BUS.Manager     imem_bus,
          OBI_BUS.Manager     dmem_bus,
          OBI_BUS.Manager     intc_bus,
          OBI_BUS.Manager     per_bus,
          OBI_BUS.Subordinate sba_bus,
          OBI_BUS.Manager     dbg_bus,
          OBI_BUS.Manager     mbx_bus,
          OBI_BUS.Manager     mgr_bus,
          OBI_BUS.Manager     rom_bus,
          OBI_BUS.Subordinate sbr_bus
);
  localparam obi_pkg::obi_cfg_t ObiCfg = obi_pkg::ObiDefaultConfig;

  localparam int unsigned NumMgr = 3;
  localparam int unsigned NumSbr = 4;
  localparam int unsigned NumDemux = 5;
  localparam int unsigned DemuxWidth = $clog2(NumDemux);

  localparam bit [NumMgr-1:0][NumSbr-1:0] Connectivity = '{
      '{1'b1, 1'b1, 1'b1, 1'b1},  // data
      '{1'b1, 1'b0, 1'b1, 1'b1},  // inst
      '{1'b1, 1'b1, 1'b1, 1'b1}  // xbar_mgr
  };

  typedef struct packed {
    int unsigned idx;
    logic [31:0] start_addr;
    logic [31:0] end_addr;
  } rule_t;

  localparam rule_t [NumSbr-1:0] CoreAddrMap = '{
      rule_t'{idx: 0, start_addr: AddrMap.dbg.base, end_addr: AddrMap.dbg.last},
      rule_t'{idx: 1, start_addr: AddrMap.imem.base, end_addr: AddrMap.imem.last},
      rule_t'{idx: 2, start_addr: AddrMap.dmem.base, end_addr: AddrMap.dmem.last},
      rule_t'{idx: 3, start_addr: AddrMap.dmem.last, end_addr: AddrMap.ext.last}
  };

  OBI_BUS sbr_ports[NumMgr] ();
  OBI_BUS mgr_ports[NumSbr] ();

  OBI_BUS int_imem ();
  OBI_BUS int_dmem ();
  OBI_BUS int_dbg ();

  OBI_BUS mux_in[2] ();
  OBI_BUS mux_out ();

  OBI_BUS demux_in ();
  OBI_BUS demux_out[NumDemux] ();

  `OBI_ASSIGN(mux_in[0], sba_bus, ObiCfg, ObiCfg)
  `OBI_ASSIGN(mux_in[1], sbr_bus, ObiCfg, ObiCfg)

  `OBI_ASSIGN_CUT(demux_in)
  `OBI_ASSIGN_CUT(mux_out)

  logic [DemuxWidth-1:0] demux_sel;

  obi_mux_intf #(
      .NumSbrPorts(2),
      .NumMaxTrans(32'd1)
  ) i_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .mgr_port  (mux_out),
      .sbr_ports (mux_in)
  );

  always_comb begin : demux_assign
    demux_sel = '0;
    unique case (demux_in_cut.addr) inside
      [AddrMap.intc.base : AddrMap.intc.last - 1]: demux_sel = 0;
      [AddrMap.dmem.last : AddrMap.intc.base - 1]: demux_sel = 1;  // Peripherals
      [AddrMap.mbx.base : AddrMap.mbx.last - 1]:   demux_sel = 3;
      [AddrMap.ext.base : AddrMap.ext.last - 1]:   demux_sel = 4;
      [AddrMap.rom.base : AddrMap.rom.last - 1]:   demux_sel = 5;
      default:                                     ;
    endcase
  end : demux_assign

  obi_demux_intf #(
      .NumMgrPorts(NumDemux),
      .NumMaxTrans(32'd1)
  ) i_demux (
      .clk_i,
      .rst_ni,
      .sbr_port_select_i(demux_sel),
      .mgr_ports(demux_out),
      .sbr_port(demux_in_cut)
  );

  `OBI_ASSIGN(sbr_ports[0], mux_out_cut, ObiCfg, ObiCfg)
  `OBI_ASSIGN(sbr_ports[1], inst_bus, ObiCfg, ObiCfg)
  `OBI_ASSIGN(sbr_ports[2], data_bus, ObiCfg, ObiCfg)

  `OBI_ASSIGN(int_dbg, mgr_ports[0], ObiCfg, ObiCfg)
  `OBI_ASSIGN(int_imem, mgr_ports[1], ObiCfg, ObiCfg)
  `OBI_ASSIGN(int_dmem, mgr_ports[2], ObiCfg, ObiCfg)
  `OBI_ASSIGN(demux_in, mgr_ports[3], ObiCfg, ObiCfg)

  // Cut memory busses, assign to top-level ports
  `OBI_ASSIGN_CUT(int_imem)
  `OBI_ASSIGN(imem_bus, int_imem_cut, ObiCfg, ObiCfg)

  `OBI_ASSIGN_CUT(int_dmem)
  `OBI_ASSIGN(dmem_bus, int_dmem_cut, ObiCfg, ObiCfg)

  `OBI_ASSIGN_CUT(int_dbg)
  `OBI_ASSIGN(dbg_bus, int_dbg_cut, ObiCfg, ObiCfg)

  // Assign demux ports
  `OBI_ASSIGN(intc_bus, demux_out[0], ObiCfg, ObiCfg)
  `OBI_ASSIGN(per_bus, demux_out[1], ObiCfg, ObiCfg)
  `OBI_ASSIGN(mbx_bus, demux_out[2], ObiCfg, ObiCfg)
  `OBI_ASSIGN(mgr_bus, demux_out[3], ObiCfg, ObiCfg)
  `OBI_ASSIGN(rom_bus, demux_out[4], ObiCfg, ObiCfg)

  zeroheti_pkg::obi_req_t [NumMgr-1:0] sbr_ports_req;
  zeroheti_pkg::obi_rsp_t [NumMgr-1:0] sbr_ports_rsp;

  zeroheti_pkg::obi_req_t [NumSbr-1:0] mgr_ports_req;
  zeroheti_pkg::obi_rsp_t [NumSbr-1:0] mgr_ports_rsp;

  for (genvar i = 0; i < NumMgr; i++) begin : gen_sbr_ports_assign
    `OBI_ASSIGN_TO_REQ(sbr_ports_req[i], sbr_ports[i], ObiCfg)
    `OBI_ASSIGN_FROM_RSP(sbr_ports[i], sbr_ports_rsp[i], ObiCfg)
  end

  for (genvar i = 0; i < NumSbr; i++) begin : gen_mgr_ports_assign
    `OBI_ASSIGN_FROM_REQ(mgr_ports[i], mgr_ports_req[i], ObiCfg)
    `OBI_ASSIGN_TO_RSP(mgr_ports_rsp[i], mgr_ports[i], ObiCfg)
  end

  obi_xbar #(
      .sbr_port_obi_req_t(zeroheti_pkg::obi_req_t),
      .sbr_port_a_chan_t (zeroheti_pkg::obi_a_chan_t),
      .sbr_port_obi_rsp_t(zeroheti_pkg::obi_rsp_t),
      .sbr_port_r_chan_t (zeroheti_pkg::obi_r_chan_t),
      .mgr_port_obi_req_t(zeroheti_pkg::obi_req_t),
      .mgr_port_obi_rsp_t(zeroheti_pkg::obi_rsp_t),
      .NumSbrPorts       (NumMgr),
      .NumMgrPorts       (NumSbr),
      .NumMaxTrans       (32'd1),
      .NumAddrRules      (NumSbr),
      .addr_map_rule_t   (rule_t),
      .UseIdForRouting   (1'b0),
      .Connectivity      (Connectivity)
  ) i_obi_xbar (
      .clk_i,
      .rst_ni,
      .testmode_i      (1'b0),
      .sbr_ports_req_i (sbr_ports_req),
      .sbr_ports_rsp_o (sbr_ports_rsp),
      .mgr_ports_req_o (mgr_ports_req),
      .mgr_ports_rsp_i (mgr_ports_rsp),
      .addr_map_i      (CoreAddrMap),
      .en_default_idx_i('0),
      .default_idx_i   ('0)
  );

endmodule : zeroheti_xbar

