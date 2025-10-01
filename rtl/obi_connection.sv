module obi_connection #(
  parameter bit Cut = 0
)(
  input logic         clk_i,
  input logic         rst_ni,
  OBI_BUS.Subordinate Src,
  OBI_BUS.Manager     Dst
);

if (Cut) begin : g_cut
  
  `OBI_TYPEDEF_ALL(sbr_port_obi, obi_pkg::ObiDefaultConfig)
  `OBI_TYPEDEF_ALL(mgr_port_obi, obi_pkg::ObiDefaultConfig)

  sbr_port_obi_req_t sbr_ports_req;
  sbr_port_obi_rsp_t sbr_ports_rsp;
  mgr_port_obi_req_t mgr_ports_req;
  mgr_port_obi_rsp_t mgr_ports_rsp;

  `OBI_ASSIGN_TO_REQ(sbr_ports_req, Src, obi_pkg::ObiDefaultConfig)
  `OBI_ASSIGN_FROM_RSP(Src, sbr_ports_rsp, obi_pkg::ObiDefaultConfig)

  `OBI_ASSIGN_FROM_REQ(Dst, mgr_ports_req, obi_pkg::ObiDefaultConfig)
  `OBI_ASSIGN_TO_RSP(mgr_ports_rsp, Dst, obi_pkg::ObiDefaultConfig)

  obi_cut #(
    .obi_a_chan_t (sbr_port_obi_a_chan_t),
    .obi_r_chan_t (sbr_port_obi_r_chan_t),
    .obi_req_t    (sbr_port_obi_req_t),
    .obi_rsp_t    (sbr_port_obi_rsp_t)
  ) i_obi_cut (
    .clk_i,
    .rst_ni,
    .sbr_port_req_i (sbr_ports_req),
    .sbr_port_rsp_o (sbr_ports_rsp),
    .mgr_port_req_o (mgr_ports_req),
    .mgr_port_rsp_i (mgr_ports_rsp)
  );

end else begin : g_no_cut
  obi_join i_join (.Src, .Dst);
end

endmodule : obi_connection

