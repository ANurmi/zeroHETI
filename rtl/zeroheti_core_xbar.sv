module zeroheti_core_xbar #()(
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


obi_demux_intf #(
  .NumMgrPorts (IfDst)
) i_if_demux (
  .clk_i,
  .rst_ni,
  .sbr_port_select_i (1'b0),
  .sbr_port          (cpu_if),
  .mgr_ports         (if_demux)
);

obi_join if_im_join ( .Src (if_demux[0]), .Dst (im_mux[0]) );

obi_mux_intf #(
  .NumSbrPorts (ImSrc)
) i_inst_mux (
  .clk_i,
  .rst_ni,
  .testmode_i(1'b0),
  .sbr_ports (im_mux),
  .mgr_port  (inst_sbr)
);

endmodule : zeroheti_core_xbar

