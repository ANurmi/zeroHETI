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


  localparam int unsigned WordsBank = NumWords / NrBanks;
  localparam int unsigned NumPorts = 1;
  localparam int unsigned Latency = 1;
  localparam SimInit = "random";

  logic               rvalid_q;
  logic [NrBanks-1:0] req_mux;
  logic [NrBanks-1:0] req_q, req_d;
  logic [        NrBanks-1:0]                we_mux;
  logic [        NrBanks-1:0][DataWidth-1:0] rdata_mux;
  logic [               31:0]                mem_addr;
  logic [               29:0]                word_addr;
  logic [$clog2(NrBanks)-1:0]                rdata_sel;

  assign mem_addr  = sbr.addr - BaseAddr;
  assign word_addr = mem_addr[31:2];

`ifndef SYNTHESIS
  initial begin
    if (NrBanks > 16) $fatal(1, "Maximun supported bank number is 16!");
  end
`endif

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      rvalid_q <= 1'b0;
      req_q    <= 'b0;
    end else begin
      rvalid_q <= sbr.req;
      req_q    <= req_d;
    end
  end

  always_comb begin : rdata_lut
    //rdata_sel = 0;
    unique case (req_q)
      1: rdata_sel = 0;
      2: rdata_sel = 1;
      4: rdata_sel = 2;
      8: rdata_sel = 3;
      16: rdata_sel = 4;
      default: rdata_sel = 0;
    endcase
  end

  assign sbr.rdata = rdata_mux[rdata_sel];

  for (genvar i = 0; i < NrBanks; i++) begin : g_banks

    localparam int unsigned LocalWordBase = i * WordsBank;
    localparam int unsigned LocalWordLast = ((i + 1) * WordsBank) - 1;
    localparam int unsigned LocalAddrWidth = (WordsBank > 32'd1) ? $clog2(WordsBank) : 32'd1;

    logic [29:0] local_word_addr;
    logic [LocalAddrWidth-1:0] sram_addr;

    assign local_word_addr = word_addr - LocalWordBase;
    assign sram_addr = local_word_addr[AddrWidth-1:0];

    always_comb begin : addr_decode

      req_mux[i] = 1'b0;
      we_mux[i]  = 1'b0;
      req_d[i]   = 'b0;

      if (mem_addr inside {[LocalWordBase : LocalWordLast]}) begin
        req_mux[i] = sbr.req;
        req_d[i]   = sbr.req;
        we_mux[i]  = sbr.we;
      end

    end

    tc_sram #(
        .NumWords (WordsBank),
        .DataWidth(DataWidth),
        .ByteWidth(ByteWidth),
        .NumPorts (NumPorts),
        .Latency  (Latency),
        .SimInit  (SimInit),
        .ImplKey  ("zeroheti_lp_sram_v0")
    ) i_sram (
        .clk_i,
        .rst_ni,
        .req_i  (req_mux[i]),
        .we_i   (we_mux[i]),
        .be_i   (sbr.be),
        .addr_i (sram_addr),
        .wdata_i(sbr.wdata),
        .rdata_o(rdata_mux[i])
    );

  end

  assign sbr.gnt    = sbr.req;
  assign sbr.rvalid = rvalid_q;

  // Tie off unused parts
  assign sbr.gntpar = 1'b0;
  assign sbr.rvalidpar = 1'b0;
  assign sbr.rid = 1'b0;
  assign sbr.r_optional = 1'b0;
  assign sbr.err    = 1'b0;

endmodule : obi_mb_sram_intf

