module obi_connection #(
  parameter bit Cut = 0
)(
  input logic         clk_i,
  input logic         rst_ni,
  OBI_BUS.Subordinate Src,
  OBI_BUS.Manager     Dst
);

if (Cut) begin : g_cut
  $fatal("Unimplemented!");
end else begin : g_no_cut
  obi_join i_join (.Src, .Dst);
end

endmodule : obi_connection

