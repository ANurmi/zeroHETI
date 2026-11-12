module uart_wrapper #(
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic [31:0] paddr_i,
    input  logic [31:0] pwdata_i,
    output logic [31:0] prdata_o,
    output logic        pslverr_o,
    output logic        pready_o,
    input  logic        pwrite_i,
    input  logic        penable_i,
    input  logic        psel_i,
    input  logic [ 3:0] pstrb_i,
    output logic        irq_o,
    input  logic        rx_i,
    output logic        tx_o
);

`ifndef FULL_UART
  mock_uart i_mock_uart (
      .clk_i,
      .rst_ni,
      .penable_i,
      .pwrite_i,
      .paddr_i,
      .psel_i,
      .pwdata_i,
      .pstrb_i,
      .prdata_o,
      .pready_o,
      .pslverr_o
  );
  assign irq_o = 1'b0;
  assign tx_o  = 1'b0;
`else

  typedef enum logic [1:0] {
    IDLE,
    RD_SETUP,
    RD_ACCESS,
    RD_DONE
  } rd_state_e;

  rd_state_e state_d, state_q;
  logic [1:0] lane_d, lane_q;
  logic [31:0] rbuf_d, rbuf_q;

  logic [31:0] rdata_local;
  logic [31:0] wdata_local;
  logic [ 2:0] addr_local;
  logic [ 1:0] addr_offs;
  logic        strb_err;

  logic uart_psel, uart_penable, uart_pready, uart_pslverr;

  logic wr_req, rd_req;
  assign wr_req = psel_i & pwrite_i;
  assign rd_req = psel_i & ~pwrite_i;

  // Write byte-lane decode: move the selected byte to lane 0
  always_comb begin
    addr_offs   = '0;
    wdata_local = '0;
    strb_err    = 1'b0;
    if (pwrite_i) begin
      unique case (pstrb_i)
        4'b0001: begin
          addr_offs   = 2'd0;
          wdata_local = {24'h0, pwdata_i[7:0]};
        end
        4'b0010: begin
          addr_offs   = 2'd1;
          wdata_local = {24'h0, pwdata_i[15:8]};
        end
        4'b0100: begin
          addr_offs   = 2'd2;
          wdata_local = {24'h0, pwdata_i[23:16]};
        end
        4'b1000: begin
          addr_offs   = 2'd3;
          wdata_local = {24'h0, pwdata_i[31:24]};
        end
        default: strb_err = 1'b1;
      endcase
    end
  end

  always_comb begin
    state_d      = state_q;
    lane_d       = lane_q;
    rbuf_d       = rbuf_q;

    uart_psel    = 1'b0;
    uart_penable = 1'b0;
    addr_local   = {paddr_i[2], addr_offs};

    pready_o     = 1'b0;
    pslverr_o    = 1'b0;
    prdata_o     = '0;

    unique case (state_q)
      IDLE: begin
        if (wr_req) begin
          if (strb_err) begin
            // Only single-byte writes are supported
            pready_o  = penable_i;
            pslverr_o = penable_i;
          end else begin
            uart_psel    = psel_i;
            uart_penable = penable_i;
            pready_o     = uart_pready;
            pslverr_o    = uart_pslverr;
          end
        end else if (rd_req & penable_i) begin
          lane_d  = '0;
          rbuf_d  = '0;
          state_d = RD_SETUP;
        end
      end
      RD_SETUP: begin
        uart_psel  = 1'b1;
        addr_local = {paddr_i[2], lane_q};
        state_d    = RD_ACCESS;
      end
      RD_ACCESS: begin
        uart_psel    = 1'b1;
        uart_penable = 1'b1;
        addr_local   = {paddr_i[2], lane_q};
        if (uart_pready) begin
          // Lanes that error (e.g. non-divisor regs while DLAB=1) read as 0
          if (!uart_pslverr) rbuf_d[8*lane_q+:8] = rdata_local[7:0];
          if (lane_q == 2'd3) begin
            state_d = RD_DONE;
          end else begin
            lane_d  = lane_q + 2'd1;
            state_d = RD_SETUP;
          end
        end
      end
      RD_DONE: begin
        pready_o = 1'b1;
        prdata_o = rbuf_q;
        state_d  = IDLE;
      end
      default: state_d = IDLE;
    endcase
  end

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      state_q <= IDLE;
      lane_q  <= '0;
      rbuf_q  <= '0;
    end else begin
      state_q <= state_d;
      lane_q  <= lane_d;
      rbuf_q  <= rbuf_d;
    end
  end

  apb_uart i_apb_uart (
      .CLK    (clk_i),
      .RSTN   (rst_ni),
      .PSEL   (uart_psel),
      .PENABLE(uart_penable),
      .PWRITE (pwrite_i),
      .PADDR  (addr_local),
      .PWDATA (wdata_local),
      .PRDATA (rdata_local),
      .PREADY (uart_pready),
      .PSLVERR(uart_pslverr),
      .INT    (irq_o),
      .CTSN   (1'b0),
      .DSRN   (1'b0),
      .DCDN   (1'b0),
      .RIN    (1'b0),
      .RTSN   (),
      .OUT1N  (),
      .OUT2N  (),
      .DTRN   (),
      .SIN    (rx_i),
      .SOUT   (tx_o)
  );

`endif



endmodule : uart_wrapper
