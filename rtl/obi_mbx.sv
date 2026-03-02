module obi_mbx #(
) (
    input  logic               clk_i,
    input  logic               rst_ni,
    output logic               irq_o,
           OBI_BUS.Subordinate obi_sbr
);

 // assign obi_sbr.gnt = obi_sbr.req;

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      obi_sbr.rvalid <= 1'b0;
      obi_sbr.gnt    <= 1'b0;
    end else begin
      obi_sbr.gnt    <= obi_sbr.req;
      obi_sbr.rvalid <= obi_sbr.gnt;
    end
  end

`ifdef MBX_SIM
  vip_mbx i_sim_mbx (
      .clk_i,
      .rst_ni,
      .req_i(obi_sbr.req),
      .we_i(obi_sbr.we),
      .irq_o,
      .addr_i(obi_sbr.addr),
      .wdata_i(obi_sbr.wdata),
      .rdata_o(obi_sbr.rdata)
  );
`else
  assign irq_o = 1'b0;
  assign obi_sbr.rdata = 32'b0;
`endif

  // Unused tie-offs
  assign obi_sbr.gntpar     = 1'b0;
  assign obi_sbr.err        = 1'b0;
  assign obi_sbr.rvalidpar  = 1'b0;
  assign obi_sbr.rid        = 1'b0;
  assign obi_sbr.r_optional = 1'b0;

endmodule : obi_mbx

