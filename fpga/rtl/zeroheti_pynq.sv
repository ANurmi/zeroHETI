module zeroheti_pynq #()(
  input  logic clk_i,
  input  logic rst_i,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o,
  input  logic uart_rx_i,
  output logic uart_tx_o
);

logic locked, top_clk;
wire  zh_rstn;

// use locked to provide active-low synchronous reset
assign zh_rstn = locked;

// top clock instance
top_clock i_top_clock (
  .reset    ( rst_i    ), // input reset
  .locked   ( locked   ), // output locked
  .clk_in1  ( clk_i    ), // input clk_in1
  .clk_out1 ( top_clk  )  // output clk_out1
);

zeroheti_top (
  clk_i     (top_clk),
  rst_ni    (zh_rstn),
  jtag_tck_i,
  jtag_tms_i,
  jtag_trst_ni,
  jtag_td_i,
  jtag_td_o,
  uart_rx_i,
  uart_tx_o
);

endmodule : zeroheti_pynq
