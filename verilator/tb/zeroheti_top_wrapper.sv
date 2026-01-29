module zeroheti_top_wrapper #(
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic jtag_tck_i,
    input  logic jtag_tms_i,
    input  logic jtag_trst_ni,
    input  logic jtag_td_i,
    output logic jtag_td_o,
    input  logic uart_rx_i,
    output logic uart_tx_o
);

  zeroheti_top i_zeroheti (
      .clk_i,
      .rst_ni,
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .uart_rx_i,
      .uart_tx_o,
      .ext_irq_i(),
      .i2c_scl_pad_i(),
      .i2c_scl_pad_o(),
      .i2c_scl_padoen_o(),
      .i2c_sda_padoen_o(),
      .i2c_sda_pad_o (),
      .i2c_sda_pad_i ()
  );

endmodule : zeroheti_top_wrapper
