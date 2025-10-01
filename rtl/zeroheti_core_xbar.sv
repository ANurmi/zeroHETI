module zeroheti_core_xbar import zeroheti_pkg::*; #()(
  input  clk_i,
  input  rst_ni,
  OBI_BUS.Subordinate cpu_if,
  OBI_BUS.Subordinate cpu_lsu,
  OBI_BUS.Subordinate sba_mgr,
  OBI_BUS.Manager     dbg_sbr,
  OBI_BUS.Manager     inst_sbr,
  OBI_BUS.Manager     data_sbr,
  OBI_BUS.Manager     apb_sbr
);

localparam int unsigned IfDst = 32'd2;
localparam int unsigned ImSrc = 32'd2;

OBI_BUS if_demux [IfDst]();
OBI_BUS im_mux   [ImSrc]();

// Controls
logic if_demux_ctrl;
assign if_demux_ctrl = (cpu_if.addr < AddrMap.imem.base);

// Demuxes

obi_demux_intf #(
  .NumMgrPorts (IfDst),
  .NumMaxTrans (32'd1)
) i_if_demux (
  .clk_i,
  .rst_ni,
  .sbr_port_select_i (if_demux_ctrl),
  .sbr_port          (cpu_if),
  .mgr_ports         (if_demux)
);

// Connections
obi_connection #(.Cut(1'b0)) i_con_if_im  (.clk_i, .rst_ni, .Src(if_demux[0]), .Dst(im_mux[0]));
//obi_connection #(.Cut(1'b0)) i_con_if_dbg (.clk_i, .rst_ni, .Src(if_demux[1]), .Dst(dbg_sbr));

// Muxes
obi_mux_intf #(
  .NumSbrPorts (ImSrc),
  .NumMaxTrans (32'd1)
) i_inst_mux (
  .clk_i,
  .rst_ni,
  .testmode_i(1'b0),
  .sbr_ports (im_mux),
  .mgr_port  (inst_sbr)
);

endmodule : zeroheti_core_xbar

