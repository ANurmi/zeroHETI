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
  localparam int unsigned NumMgr = 3;
  localparam int unsigned NumSbr = 6;

  localparam bit [NumMgr-1:0][NumSbr-1:0] Connectivity = '{
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1},
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1},
      '{1'b1, 1'b1, 1'b1, 1'b1, 1'b1, 1'b1}
  };

  typedef struct packed {
    int unsigned idx;
    logic [31:0] start_addr;
    logic [31:0] end_addr;
  } rule_t;

  localparam rule_t [NumSbr-1:0] CoreAddrMap = '{
      rule_t'{idx: 0, start_addr: 0, end_addr: 0},
      rule_t'{idx: 0, start_addr: 0, end_addr: 0},
      rule_t'{idx: 0, start_addr: 0, end_addr: 0},
      rule_t'{idx: 0, start_addr: 0, end_addr: 0},
      rule_t'{idx: 0, start_addr: 0, end_addr: 0},
      rule_t'{idx: 0, start_addr: 0, end_addr: 0}
  };

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

  typedef struct packed {
    logic [31:0] addr;
    logic        we;
    logic [3:0]  be;
    logic [31:0] wdata;
  } a_chan_t;

  typedef struct packed {
    logic [31:0] rdata;
    logic err;
  } r_chan_t;

  obi_xbar #(
      .sbr_port_obi_req_t(),
      .sbr_port_a_chan_t (a_chan_t),
      .sbr_port_obi_rsp_t(),
      .sbr_port_r_chan_t (r_chan_t),
      .mgr_port_obi_req_t(),
      .mgr_port_obi_rsp_t(),
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
      .sbr_ports_req_i (),
      .sbr_ports_rsp_o (),
      .mgr_ports_req_o (),
      .mgr_ports_rsp_i (),
      .addr_map_i      (CoreAddrMap),
      .en_default_idx_i('0),
      .default_idx_i   ('0)
  );


endmodule : zeroheti_xbar

