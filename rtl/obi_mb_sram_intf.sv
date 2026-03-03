module obi_mb_sram_intf #(
    parameter  int unsigned NrBanks   = 32'd1,
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


endmodule : obi_mb_sram_intf

