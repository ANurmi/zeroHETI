module apb_zhic #()(
  input  logic        clk_i,
  input  logic        rst_ni,
  input  logic        penable_i,
  input  logic        pwrite_i,
  input  logic [31:0] paddr_i,
  input  logic        psel_i,
  input  logic [31:0] pwdata_i,
  output logic [31:0] prdata_o,
  output logic        pready_o,
  output logic        pslverr_o
);
endmodule : apb_zhic

