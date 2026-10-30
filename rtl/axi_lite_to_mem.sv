// AXI4-Lite to fixed-latency memory port
//
// Converts an `AXI_LITE.Slave` port into a simple single-port memory request
// interface with no grant and no read-valid: every request is accepted in the
// cycle it is issued and read data is returned exactly one cycle later. This
// matches e.g. `tc_sram` (Latency = 1) and the riscv-dbg `dm_top` slave port.
//
// Memory port:
//   mem_req_o    request (one access per cycle, read or write)
//   mem_we_o     write enable
//   mem_addr_o   full AXI byte address (no translation)
//   mem_wdata_o  write data
//   mem_be_o     byte enables (write strobes; '0 on reads)
//   mem_rdata_i  read data, valid in the cycle after a read request
//
// No combinational paths through the module
// -----------------------------------------
// aw_ready, w_ready, ar_ready, b_valid and r_valid are driven directly from
// flip-flops, so the module can sit behind an `axi_lite_xbar` with
// `LatencyMode = NO_LATENCY` without forming a combinational loop.
//
// Each of AW, W and AR has a one-entry holding register. A channel's ready is
// high when its holding register is empty, which it is in the steady state:
//   - A request is served straight from the bus in the cycle it is accepted if
//     its response channel can take the response.
//   - Otherwise it is parked in the holding register and served from there.
//   - AW and W are accepted independently; the write happens once both the
//     address and the data are available.
//
// Timing:
//   - Without back-pressure, a request accepted in cycle N gets its response
//     in cycle N+1, and each channel can accept a new request every cycle.
//   - If a read and a write are both pending in the same cycle, a round-robin
//     bit picks one; the other waits one cycle in its holding register.
//   - If r_ready is low, mem_rdata_i is captured so r_data stays stable.
//   - All responses are OKAY.

module axi_lite_to_mem #(
    /// AXI address width. Must match the `AXI_LITE` interface.
    parameter  int unsigned AddrWidth = 32'd32,
    /// AXI data width. Must match the `AXI_LITE` interface.
    parameter  int unsigned DataWidth = 32'd32,
    /// Dependent parameters, do not override.
    localparam type         addr_t    = logic [AddrWidth-1:0],
    localparam type         data_t    = logic [DataWidth-1:0],
    localparam type         strb_t    = logic [DataWidth/8-1:0]
) (
    input  logic          clk_i,
    input  logic          rst_ni,
           AXI_LITE.Slave slv,
    output logic          mem_req_o,
    output logic          mem_we_o,
    output addr_t         mem_addr_o,
    output data_t         mem_wdata_o,
    output strb_t         mem_be_o,
    input  data_t         mem_rdata_i
);

  // ---------------------------------------------------------------------------
  // Holding registers; ready = holding register empty (flop-driven)
  // ---------------------------------------------------------------------------
  logic aw_held_q, w_held_q, ar_held_q;
  addr_t aw_addr_q, ar_addr_q;
  data_t w_data_q;
  strb_t w_strb_q;

  assign slv.aw_ready = !aw_held_q;
  assign slv.w_ready  = !w_held_q;
  assign slv.ar_ready = !ar_held_q;

  logic aw_hs, w_hs, ar_hs;
  assign aw_hs = slv.aw_valid && !aw_held_q;
  assign w_hs  = slv.w_valid && !w_held_q;
  assign ar_hs = slv.ar_valid && !ar_held_q;

  // Current request: from the holding register if full, else from the bus.
  logic have_aw, have_w, have_ar;
  assign have_aw = aw_held_q || aw_hs;
  assign have_w  = w_held_q || w_hs;
  assign have_ar = ar_held_q || ar_hs;

  addr_t wr_addr, rd_addr;
  data_t wr_data;
  strb_t wr_strb;
  assign wr_addr = aw_held_q ? aw_addr_q : slv.aw_addr;
  assign wr_data = w_held_q ? w_data_q : slv.w_data;
  assign wr_strb = w_held_q ? w_strb_q : slv.w_strb;
  assign rd_addr = ar_held_q ? ar_addr_q : slv.ar_addr;

  // ---------------------------------------------------------------------------
  // Arbitration (internal only; never feeds a ready signal)
  // ---------------------------------------------------------------------------
  logic b_valid_q, r_valid_q;
  logic b_free, r_free;
  logic wr_req, rd_req, conflict, wr_gnt, rd_gnt;
  logic prio_wr_q;  // on a read/write conflict: 1 = write wins, 0 = read wins

  // A response channel is free if it holds nothing or is handshaking this cycle.
  assign b_free   = !b_valid_q || slv.b_ready;
  assign r_free   = !r_valid_q || slv.r_ready;

  assign wr_req   = have_aw && have_w && b_free;
  assign rd_req   = have_ar && r_free;

  assign conflict = wr_req && rd_req;
  assign wr_gnt   = wr_req && (!conflict || prio_wr_q);
  assign rd_gnt   = rd_req && (!conflict || !prio_wr_q);

  // ---------------------------------------------------------------------------
  // Memory port
  // ---------------------------------------------------------------------------
  assign mem_req_o   = wr_gnt || rd_gnt;
  assign mem_we_o    = wr_gnt;
  assign mem_addr_o  = wr_gnt ? wr_addr : rd_addr;
  assign mem_wdata_o = wr_data;
  assign mem_be_o    = wr_gnt ? wr_strb : '0;

  // ---------------------------------------------------------------------------
  // Sequential state
  // ---------------------------------------------------------------------------
  logic  r_hold_q;  // r_data_q holds the read data (R channel stalled)
  data_t r_data_q;

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      aw_held_q <= 1'b0;
      w_held_q  <= 1'b0;
      ar_held_q <= 1'b0;
      aw_addr_q <= '0;
      ar_addr_q <= '0;
      w_data_q  <= '0;
      w_strb_q  <= '0;
      b_valid_q <= 1'b0;
      r_valid_q <= 1'b0;
      r_hold_q  <= 1'b0;
      r_data_q  <= '0;
      prio_wr_q <= 1'b0;
    end else begin
      // Park an accepted request that was not served; release it once served.
      if (wr_gnt) begin
        aw_held_q <= 1'b0;
        w_held_q  <= 1'b0;
      end else begin
        if (aw_hs) begin
          aw_held_q <= 1'b1;
          aw_addr_q <= slv.aw_addr;
        end
        if (w_hs) begin
          w_held_q <= 1'b1;
          w_data_q <= slv.w_data;
          w_strb_q <= slv.w_strb;
        end
      end

      if (rd_gnt) begin
        ar_held_q <= 1'b0;
      end else if (ar_hs) begin
        ar_held_q <= 1'b1;
        ar_addr_q <= slv.ar_addr;
      end

      // Write response
      if (wr_gnt) begin
        b_valid_q <= 1'b1;
      end else if (slv.b_ready) begin
        b_valid_q <= 1'b0;
      end

      // Read response. mem_rdata_i is only guaranteed in the cycle after the
      // request, so if the manager stalls we capture it once.
      if (rd_gnt) begin
        r_valid_q <= 1'b1;
        r_hold_q  <= 1'b0;
      end else if (slv.r_ready) begin
        r_valid_q <= 1'b0;
        r_hold_q  <= 1'b0;
      end else if (r_valid_q && !r_hold_q) begin
        r_hold_q <= 1'b1;
        r_data_q <= mem_rdata_i;
      end

      // Round-robin: after a conflict, the other side gets priority.
      if (conflict) begin
        prio_wr_q <= !prio_wr_q;
      end
    end
  end

  assign slv.b_valid = b_valid_q;
  assign slv.b_resp  = axi_pkg::RESP_OKAY;

  assign slv.r_valid = r_valid_q;
  assign slv.r_data  = r_hold_q ? r_data_q : mem_rdata_i;
  assign slv.r_resp  = axi_pkg::RESP_OKAY;

  // ---------------------------------------------------------------------------
  // Parameter checks
  // ---------------------------------------------------------------------------
`ifndef SYNTHESIS
  initial begin : p_param_check
    if ($bits(slv.aw_addr) != AddrWidth)
      $fatal(1, "AddrWidth (%0d) does not match the AXI_LITE interface (%0d)", AddrWidth,
             $bits(slv.aw_addr));
    if ($bits(slv.w_data) != DataWidth)
      $fatal(1, "DataWidth (%0d) does not match the AXI_LITE interface (%0d)", DataWidth,
             $bits(slv.w_data));
    if (DataWidth < 8 || DataWidth % 8 != 0)
      $fatal(1, "DataWidth must be a non-zero multiple of 8");
  end
`endif

endmodule : axi_lite_to_mem
