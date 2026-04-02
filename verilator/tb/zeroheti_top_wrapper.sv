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

  logic i2c_sda_vip_dut;
  logic i2c_sda_dut_vip;
  logic i2c_scl_vip_dut;
  logic i2c_scl_dut_vip;

  logic [3:0] i2c_vip_irqs;

  vip_i2c i_vip_i2c (
      .clk_i,
      .rst_ni,
      .sda_i(i2c_sda_dut_vip),
      .sda_o(i2c_sda_vip_dut),
      .scl_i(i2c_scl_dut_vip),
      .scl_o(i2c_scl_vip_dut),
      .irq_o(i2c_vip_irqs)  /* idx 26-29*/
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
      .axil_aw_addr_i  (),
      .axil_aw_valid_i (),
      .axil_aw_ready_o (),
      .axil_aw_prot_i  (),
      .axil_w_valid_i  (),
      .axil_w_strb_i   (),
      .axil_w_ready_o  (),
      .axil_w_data_i   (),
      .axil_b_resp_o   (),
      .axil_b_valid_o  (),
      .axil_b_ready_i  (),
      .axil_ar_addr_i  (),
      .axil_ar_valid_i (),
      .axil_ar_ready_o (),
      .axil_ar_prot_i  (),
      .axil_r_data_o   (),
      .axil_r_resp_o   (),
      .axil_r_valid_o  (),
      .axil_r_ready_i  (),
      .ext_irq_i       ({1'h0, i2c_vip_irqs}),
      .i2c_scl_pad_i   (i2c_scl_vip_dut),
      .i2c_scl_pad_o   (  /*NC*/),
      .i2c_scl_padoen_o(i2c_scl_dut_vip),
      .i2c_sda_padoen_o(i2c_sda_dut_vip),
      .i2c_sda_pad_o   (  /*NC*/),
      .i2c_sda_pad_i   (i2c_sda_vip_dut)
  );

endmodule : zeroheti_top_wrapper
