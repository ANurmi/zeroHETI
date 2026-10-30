// AXI4-Lite SRAM wrapper (single-port, registered ready signals)
//
// A single-port SRAM of `NumWords` words behind an `AXI_LITE.Slave` port. The
// memory is physically split into `NumBanks` single-port `tc_sram` macros
// (pulp-platform/tech_cells_generic, fixed latency of 1 cycle) of
// NumWords/NumBanks words each. Banking only splits the capacity: exactly one
// access (read or write) is performed per cycle, so the module behaves exactly
// like one single-port SRAM of `NumWords` words.
//
// Address map: contiguous (the bank select is the top of the word address).
//   addr[ByteOffW-1:0]                      byte offset within a word (ignored, use strobes)
//   addr[ByteOffW +: RowAddrW]              word address inside the bank
//   addr[ByteOffW+RowAddrW +: BankSelW]     bank select
//   upper bits                              ignored (the memory aliases)
//
// The AXI4-Lite protocol handling is done by `axi_lite_to_mem`, see there for
// timing details. In short: no combinational paths through the module (all
// ready/valid outputs are flop-driven), a request accepted in cycle N is
// answered in cycle N+1 without back-pressure, read/write conflicts are
// resolved round-robin, and all responses are OKAY.

module axi_lite_sram #(
    /// AXI address width. Must match the `AXI_LITE` interface.
    parameter int unsigned AddrWidth   = 32'd32,
    /// AXI data width (= SRAM word width). Must match the `AXI_LITE` interface.
    parameter int unsigned DataWidth   = 32'd32,
    /// Total number of words in the memory. Must be a power of two.
    parameter int unsigned NumWords    = 32'd1024,
    /// Number of physical SRAM macros the memory is split into. Must be a power
    /// of two and at most `NumWords`. Each macro holds NumWords/NumBanks words.
    parameter int unsigned NumBanks    = 32'd1,
    /// Simulation init value of the SRAM model ("zeros", "ones", "random", "none").
    parameter              SimInit     = "none",
    /// Print the SRAM model configuration at simulation start.
    parameter bit          PrintSimCfg = 1'b0
) (
    input logic clk_i,
    input logic rst_ni,
    AXI_LITE.Slave slv
);

  // ---------------------------------------------------------------------------
  // Derived parameters and types
  // ---------------------------------------------------------------------------
  localparam int unsigned Latency = 32'd1;  // fixed by axi_lite_to_mem
  localparam int unsigned StrbWidth = DataWidth / 8;
  localparam int unsigned ByteOffW = $clog2(StrbWidth);
  localparam int unsigned BankSelW = (NumBanks > 1) ? $clog2(NumBanks) : 0;
  localparam int unsigned BankIdxW = (NumBanks > 1) ? $clog2(NumBanks) : 1;
  localparam int unsigned BankWords = NumWords / NumBanks;
  localparam int unsigned RowAddrW = (BankWords > 1) ? $clog2(BankWords) : 1;
  localparam int unsigned RowSelW = (BankWords > 1) ? $clog2(BankWords) : 0;

  typedef logic [AddrWidth-1:0] addr_t;
  typedef logic [DataWidth-1:0] data_t;
  typedef logic [StrbWidth-1:0] strb_t;
  typedef logic [BankIdxW-1:0] bank_idx_t;
  typedef logic [RowAddrW-1:0] row_addr_t;

  function automatic bank_idx_t bank_of(input addr_t addr);
    if (NumBanks == 1) return '0;
    return addr[ByteOffW+RowSelW+:BankIdxW];
  endfunction

  function automatic row_addr_t row_of(input addr_t addr);
    if (BankWords == 1) return '0;
    return addr[ByteOffW+:RowAddrW];
  endfunction

  // ---------------------------------------------------------------------------
  // AXI4-Lite -> single-port memory
  // ---------------------------------------------------------------------------
  logic mem_req, mem_we;
  addr_t mem_addr;
  data_t mem_wdata, mem_rdata;
  strb_t mem_be;

  axi_lite_to_mem #(
      .AddrWidth(AddrWidth),
      .DataWidth(DataWidth)
  ) i_axi_lite_to_mem (
      .clk_i,
      .rst_ni,
      .slv,
      .mem_req_o  (mem_req),
      .mem_we_o   (mem_we),
      .mem_addr_o (mem_addr),
      .mem_wdata_o(mem_wdata),
      .mem_be_o   (mem_be),
      .mem_rdata_i(mem_rdata)
  );

  // ---------------------------------------------------------------------------
  // SRAM banks
  // ---------------------------------------------------------------------------
  // The access is broadcast to all banks; only the addressed bank gets the
  // request. The read data is muxed from the bank of the last read.
  bank_idx_t mem_bank, r_bank_q;
  data_t [NumBanks-1:0] bank_rdata;

  assign mem_bank = bank_of(mem_addr);

  for (genvar i = 0; i < NumBanks; i++) begin : g_banks
    tc_sram #(
        .NumWords   (BankWords),
        .DataWidth  (DataWidth),
        .ByteWidth  (32'd8),
        .NumPorts   (32'd1),
        .Latency    (Latency),
        .SimInit    (SimInit),
        .PrintSimCfg(PrintSimCfg)
    ) i_sram (
        .clk_i,
        .rst_ni,
        .req_i  (mem_req && (mem_bank == bank_idx_t'(i))),
        .we_i   (mem_we),
        .addr_i (row_of(mem_addr)),
        .wdata_i(mem_wdata),
        .be_i   (mem_be),
        .rdata_o(bank_rdata[i])
    );
  end

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      r_bank_q <= '0;
    end else if (mem_req && !mem_we) begin
      r_bank_q <= mem_bank;
    end
  end

  assign mem_rdata = bank_rdata[r_bank_q];

  // ---------------------------------------------------------------------------
  // Parameter checks
  // ---------------------------------------------------------------------------
`ifndef SYNTHESIS
  initial begin : p_param_check
    if ($bits(slv.aw_addr) != AddrWidth)
      $fatal(
          1,
          "AddrWidth (%0d) does not match the AXI_LITE interface (%0d)",
          AddrWidth,
          $bits(
              slv.aw_addr
          )
      );
    if ($bits(slv.w_data) != DataWidth)
      $fatal(
          1,
          "DataWidth (%0d) does not match the AXI_LITE interface (%0d)",
          DataWidth,
          $bits(
              slv.w_data
          )
      );
    if (DataWidth < 8 || DataWidth % 8 != 0)
      $fatal(1, "DataWidth must be a non-zero multiple of 8");
    if (NumBanks == 0 || (NumBanks & (NumBanks - 1)) != 0)
      $fatal(1, "NumBanks (%0d) must be a power of two", NumBanks);
    if (NumWords == 0 || (NumWords & (NumWords - 1)) != 0)
      $fatal(1, "NumWords (%0d) must be a power of two", NumWords);
    if (NumBanks > NumWords)
      $fatal(1, "NumBanks (%0d) must not exceed NumWords (%0d)", NumBanks, NumWords);
    if (ByteOffW + RowSelW + BankSelW > AddrWidth)
      $fatal(1, "AddrWidth (%0d) too small for %0d words", AddrWidth, NumWords);
  end
`endif

endmodule
