module obi_sram_intf #(
    parameter  int unsigned NumWords  = 32'd1024,
    parameter  int unsigned DataWidth = 32'd32,
    parameter  int unsigned BaseAddr  = 32'h0,
    localparam int unsigned ByteWidth = 32'h8,
    localparam int unsigned AddrWidth = (NumWords > 32'd1) ? $clog2(NumWords) : 32'd1,
    localparam int unsigned BeWidth   = (DataWidth + ByteWidth - 32'd1) / ByteWidth
) (
    input logic clk_i,
    input logic rst_ni,
    OBI_BUS.Subordinate sbr
);

  localparam int unsigned NumPorts = 1;
  localparam int unsigned Latency = 1;
  localparam SimInit = "random";

  logic [31:0] offset_addr;
  logic [29:0] word_addr;
  logic [AddrWidth-1:0] sram_addr;

  logic rvalid_q;

  assign offset_addr = sbr.addr - BaseAddr;
  assign word_addr   = offset_addr[31:2];
  assign sram_addr   = word_addr[AddrWidth-1:0];

  // Assume zero-wait state memory -> rvalid follows req immediately
  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      rvalid_q <= 1'b0;
    end else begin
      rvalid_q <= sbr.req;
    end
  end

  tc_sram #(
      .NumWords (NumWords),
      .DataWidth(DataWidth),
      .ByteWidth(ByteWidth),
      .NumPorts (NumPorts),
      .Latency  (Latency),
      .SimInit  (SimInit),
      .ImplKey  ("zeroheti_lp_sram")
  ) i_sram (
      .clk_i,
      .rst_ni,
      .req_i  (sbr.req),
      .we_i   (sbr.we),
      .be_i   (sbr.be),
      .addr_i (sram_addr),
      .wdata_i(sbr.wdata),
      .rdata_o(sbr.rdata)
  );

  assign sbr.gnt    = sbr.req;
  assign sbr.rvalid = rvalid_q;

  // Tie off unused parts
  assign sbr.gntpar = 1'b0;
  assign sbr.rvalidpar = 1'b0;
  //assign sbr.rready = 1'b0;
  //assign sbr.rreadypar = 1'b0;
  assign sbr.rid = 1'b0;
  assign sbr.r_optional = 1'b0;
  assign sbr.err    = 1'b0;

endmodule : obi_sram_intf

