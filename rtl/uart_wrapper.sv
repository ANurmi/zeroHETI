module uart_wrapper #(
) (
    input  logic     clk_i,
    input  logic     rst_ni,
           APB.Slave apb_sbr,
    output logic     irq_o,
    input  logic     rx_i,
    output logic     tx_o
);


`ifndef FULL_UART
  mock_uart i_mock_uart (
      .clk_i,
      .rst_ni,
      .penable_i(apb_sbr.penable),
      .pwrite_i (apb_sbr.pwrite),
      .paddr_i  (apb_sbr.paddr),
      .psel_i   (apb_sbr.psel),
      .pwdata_i (apb_sbr.pwdata),
      .pstrb_i  (apb_sbr.pstrb),
      .prdata_o (apb_sbr.prdata),
      .pready_o (apb_sbr.pready),
      .pslverr_o(apb_sbr.pslverr)
  );
  assign irq_o  = 1'b0;
  assign tx_o   = 1'b0;
`else
  logic [31:0] rdata_local;
  logic [31:0] wdata_local;
  logic [ 2:0] addr_local;
  logic [ 2:0] addr_offs;

  assign addr_local = apb_sbr.paddr[2:0] + addr_offs;

  always_comb begin
    addr_offs      = 0;
    wdata_local    = 0;
    apb_sbr.prdata = 0;
    unique case (apb_sbr.pstrb)
      4'b0001: begin
        addr_offs = 0;
        apb_sbr.prdata = {24'h0, rdata_local[7:0]};
        wdata_local = {24'h0, apb_sbr.pwdata[7:0]};
      end
      4'b0010: begin
        addr_offs = 1;
        apb_sbr.prdata = {16'h0, rdata_local[7:0], 8'h0};
        wdata_local = {24'h0, apb_sbr.pwdata[15:8]};
      end
      4'b0100: begin
        addr_offs = 2;
        apb_sbr.prdata = {8'h0, rdata_local[7:0], 16'h0};
        wdata_local = {24'h0, apb_sbr.pwdata[23:16]};
      end
      4'b1000: begin
        addr_offs = 3;
        apb_sbr.prdata = {rdata_local[7:0], 24'h0};
        wdata_local = {24'h0, apb_sbr.pwdata[31:24]};
      end
      default: ;
    endcase
  end

  apb_uart i_apb_uart (
      .CLK    (clk_i),
      .RSTN   (rst_ni),
      .PSEL   (apb_sbr.psel),
      .PENABLE(apb_sbr.penable),
      .PWRITE (apb_sbr.pwrite),
      .PADDR  (addr_local),
      .PWDATA (wdata_local),
      .PRDATA (rdata_local),
      .PREADY (apb_sbr.pready),
      .PSLVERR(apb_sbr.pslverr),
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
