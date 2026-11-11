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
  logic [31:0] rdata_local;
  logic [31:0] wdata_local;
  logic [ 2:0] addr_local;
  logic [ 2:0] addr_offs;
  logic        strb_err;
  logic [ 7:0] rdata_byte;

  assign addr_local = paddr_i[2:0] + addr_offs;
  assign prdata_o   = rdata_local << 8 * 32'(addr_offs);

  always_comb begin
    addr_offs   = 0;
    wdata_local = 0;
    strb_err    = 1'b0;
    unique case (pstrb_i)
      4'b0001: begin
        addr_offs   = 0;
        wdata_local = {24'h0, pwdata_i[7:0]};
      end
      4'b0010: begin
        addr_offs   = 1;
        wdata_local = {24'h0, pwdata_i[15:8]};
      end
      4'b0100: begin
        addr_offs   = 2;
        wdata_local = {24'h0, pwdata_i[23:16]};
      end
      4'b1000: begin
        addr_offs   = 3;
        wdata_local = {24'h0, pwdata_i[31:24]};
      end
      default: if (pwrite_i) strb_err = 1'b1;
    endcase
  end

  apb_uart i_apb_uart (
      .CLK    (clk_i),
      .RSTN   (rst_ni),
      .PSEL   (psel_i),
      .PENABLE(penable_i),
      .PWRITE (pwrite_i),
      .PADDR  (addr_local),
      .PWDATA (pwdata_i),
      .PRDATA (rdata_local),
      .PREADY (pready_o),
      .PSLVERR(pslverr_o),
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
