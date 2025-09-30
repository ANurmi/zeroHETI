module obi_sram_intf #(
  parameter  int unsigned NumWords  = 32'd1024,
  parameter  int unsigned DataWidth = 32'd32,
  parameter  int unsigned BaseAddr  = 32'h0,
  localparam int unsigned AddrWidth = (NumWords > 32'd1) ? $clog2(NumWords) : 32'd1,
  localparam int unsigned BeWidth   = (DataWidth + ByteWidth - 32'd1) / ByteWidth
)(
  input logic clk_i,
  input logic rst_ni,
  OBI_BUS.Subordinate sbr
);

localparam int unsigned ByteWidth = 8;
localparam int unsigned NumPorts  = 1;
localparam int unsigned Latency   = 1;
localparam string       SimInit   = "random";

logic [31:0] offset_addr;
logic [29:0] word_addr;
logic [AddrWidth-1:0] sram_addr;

assign offset_addr = sbr.addr - BaseAddr;
assign word_addr   = offset_addr[31:2];
assign sram_addr   = word_addr[AddrWidth-1:0];

tc_sram #(
  .NumWords  (NumWords),
  .DataWidth (DataWidth),
  .ByteWidth (ByteWidth),
  .NumPorts  (NumPorts),
  .Latency   (Latency),
  .SimInit   (SimInit)
) i_sram (
  .clk_i,
  .rst_ni,
  .req_i   (sbr.req),
  .we_i    (sbr.we),
  .be_i    (sbr.be),
  .addr_i  (sram_addr),
  .wdata_i (sbr.wdata),
  .rdata_o (sbr.rdata)
);

endmodule : obi_sram_intf

