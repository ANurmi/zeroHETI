`include "obi/assign.svh"

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
          OBI_BUS.Manager     mbx_bus
);
  localparam obi_pkg::obi_cfg_t ObiCfg = obi_pkg::ObiDefaultConfig;

  localparam int unsigned NumMgr = 3;
  localparam int unsigned NumSbr = 6;

  localparam bit [NumMgr-1:0][NumSbr-1:0] Connectivity = '{
            '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1},
      '{1'b0, 1'b0, 1'b0, 1'b0, 1'b1, 1'b1},
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1}
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
      rule_t'{idx: 3, start_addr: AddrMap.intc.base, end_addr: AddrMap.intc.last},
      rule_t'{idx: 4, start_addr: AddrMap.dbg.last, end_addr: AddrMap.imem.base},
      rule_t'{idx: 5, start_addr: AddrMap.ext.base, end_addr: AddrMap.ext.last}
  };

  OBI_BUS sbr_ports[NumMgr] ();
  OBI_BUS mgr_ports[NumSbr] ();

  OBI_BUS sba_bus_cut ();
  // input SBA cut
  obi_connection #(
      .Cut(1'b1)
  ) i_cut_sba (
      .clk_i,
      .rst_ni,
      .obi_s(sba_bus),
      .obi_m(sba_bus_cut)
  );

  `OBI_ASSIGN(sbr_ports[0], sba_bus_cut, ObiCfg, ObiCfg)
  `OBI_ASSIGN(sbr_ports[1], inst_bus, ObiCfg, ObiCfg)
  `OBI_ASSIGN(sbr_ports[2], data_bus, ObiCfg, ObiCfg)

  `OBI_ASSIGN(dbg_bus, mgr_ports[0], ObiCfg, ObiCfg)
  `OBI_ASSIGN(imem_bus, mgr_ports[1], ObiCfg, ObiCfg)
  `OBI_ASSIGN(dmem_bus, mgr_ports[2], ObiCfg, ObiCfg)
  `OBI_ASSIGN(intc_bus, mgr_ports[3], ObiCfg, ObiCfg)
  `OBI_ASSIGN(per_bus, mgr_ports[4], ObiCfg, ObiCfg)
  `OBI_ASSIGN(mbx_bus, mgr_ports[5], ObiCfg, ObiCfg)

  typedef struct packed {
    logic [31:0] addr;
    logic        we;
    logic [3:0]  be;
    logic [31:0] wdata;
    logic        aid;
    logic        a_optional;
  } a_chan_t;

  typedef struct packed {
    logic [31:0] rdata;
    logic rid;
    logic err;
    logic r_optional;
  } r_chan_t;

  typedef struct packed {
    a_chan_t a;
    logic req;
  } req_t;

  typedef struct packed {
    r_chan_t r;
    logic gnt;
    logic rvalid;
  } rsp_t;

  req_t [NumMgr-1:0] sbr_ports_req;
  rsp_t [NumMgr-1:0] sbr_ports_rsp;

  req_t [NumSbr-1:0] mgr_ports_req;
  rsp_t [NumSbr-1:0] mgr_ports_rsp;


  for (genvar i = 0; i < NumMgr; i++) begin : gen_sbr_ports_assign
    `OBI_ASSIGN_TO_REQ(sbr_ports_req[i], sbr_ports[i], ObiCfg)
    `OBI_ASSIGN_FROM_RSP(sbr_ports[i], sbr_ports_rsp[i], ObiCfg)
  end

  for (genvar i = 0; i < NumSbr; i++) begin : gen_mgr_ports_assign
    `OBI_ASSIGN_FROM_REQ(mgr_ports[i], mgr_ports_req[i], ObiCfg)
    `OBI_ASSIGN_TO_RSP(mgr_ports_rsp[i], mgr_ports[i], ObiCfg)
  end


  obi_xbar #(
      .sbr_port_obi_req_t(req_t),
      .sbr_port_a_chan_t (a_chan_t),
      .sbr_port_obi_rsp_t(rsp_t),
      .sbr_port_r_chan_t (r_chan_t),
      .mgr_port_obi_req_t(req_t),
      .mgr_port_obi_rsp_t(rsp_t),
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

