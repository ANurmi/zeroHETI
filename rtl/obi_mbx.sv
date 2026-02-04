module obi_mbx #(
) (
    input logic clk_i,
    input logic rst_ni,
    output logic irq_o,
    OBI_BUS.Subordinate obi_sbr
);

`ifdef MBX_SIM
  vip_mbx i_sim_mbx ();
`endif

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      obi_sbr.rvalid <= 1'b0;
    end else begin
      obi_sbr.rvalid <= obi_sbr.req;
    end
  end

  assign obi_sbr.gnt        = obi_sbr.req;
  assign obi_sbr.gntpar     = 1'b0;
  assign obi_sbr.err        = 1'b0;
  //assign obi_sbr.rvalid     = 1'b0;
  assign obi_sbr.rdata      = 32'b0;
  assign obi_sbr.rvalidpar  = 1'b0;
  assign obi_sbr.rid        = 1'b0;
  assign obi_sbr.r_optional = 1'b0;

endmodule : obi_mbx

