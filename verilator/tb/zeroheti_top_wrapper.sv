module zeroheti_top_wrapper #(
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic jtag_tck_i,
    input  logic jtag_tms_i,
    input  logic jtag_trst_ni,
    input  logic jtag_td_i,
    output logic jtag_td_o
);

  logic i2c_sda_vip_dut;
  logic i2c_sda_dut_vip;
  logic i2c_scl_vip_dut;
  logic i2c_scl_dut_vip;

  logic [3:0] i2c_vip_irqs;

  logic dut_uart_rx;
  logic dut_uart_tx;

  vip_i2c i_vip_i2c (
      .clk_i,
      .rst_ni,
      .sda_i(i2c_sda_dut_vip),
      .sda_o(i2c_sda_vip_dut),
      .scl_i(i2c_scl_dut_vip),
      .scl_o(i2c_scl_vip_dut),
      .irq_o(i2c_vip_irqs)  /* idx 26-29*/
  );

  vip_uart i_vip_uart (
      .clk_i,
      .rst_ni,
      // flip rx-tx notation
      .uart_rx_i(dut_uart_tx),
      .uart_tx_o(dut_uart_rx)
  );

  zeroheti_top i_zeroheti (
      .clk_i,
      .rst_ni,
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .uart_rx_i       (dut_uart_rx),
      .uart_tx_o       (dut_uart_tx),
      .ext_irq_i       ({1'h0, i2c_vip_irqs}),
      .i2c_scl_pad_i   (i2c_scl_vip_dut),
      .i2c_scl_pad_o   (  /*NC*/),
      .i2c_scl_padoen_o(i2c_scl_dut_vip),
      .i2c_sda_padoen_o(i2c_sda_dut_vip),
      .i2c_sda_pad_o   (  /*NC*/),
      .i2c_sda_pad_i   (i2c_sda_vip_dut)
  );

endmodule : zeroheti_top_wrapper
