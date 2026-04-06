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
  logic [3:0] i2c_irqs;

  logic aw_valid, aw_ready;
  logic ar_valid, ar_ready;
  logic [31:0] aw_addr, ar_addr;
  logic [31:0] w_data, r_data;
  logic [3:0] w_strb;
  logic [1:0] r_resp, b_resp;
  logic [2:0] aw_prot, ar_prot;
  logic w_valid, w_ready;
  logic b_valid, b_ready;
  logic r_valid, r_ready;

  vip_zeroheti_top i_vip_zeroheti_top (
      .clk_i,
      .rst_ni,
      .sda_i     (i2c_sda_dut_vip),
      .sda_o     (i2c_sda_vip_dut),
      .scl_i     (i2c_scl_dut_vip),
      .scl_o     (i2c_scl_vip_dut),
      .i2c_irq_o (i2c_irqs),         /* idx 26-29*/
      .aw_addr_o (aw_addr),
      .aw_valid_o(aw_valid),
      .aw_ready_i(aw_ready),
      .aw_prot_o (aw_prot),
      .w_valid_o (w_valid),
      .w_strb_o  (w_strb),
      .w_ready_i (w_ready),
      .w_data_o  (w_data),
      .b_resp_i  (b_resp),
      .b_valid_i (b_valid),
      .b_ready_o (b_ready),
      .ar_addr_o (ar_addr),
      .ar_valid_o(ar_valid),
      .ar_ready_i(ar_ready),
      .ar_prot_o (ar_prot),
      .r_data_i  (r_data),
      .r_resp_i  (r_resp),
      .r_valid_i (r_valid),
      .r_ready_o (r_ready)
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
      .axil_aw_valid_i (aw_valid),
      .axil_aw_ready_o (aw_ready),
      .axil_aw_prot_i  (),
      .axil_w_valid_i  (w_valid),
      .axil_w_strb_i   (),
      .axil_w_ready_o  (w_ready),
      .axil_w_data_i   (),
      .axil_b_resp_o   (),
      .axil_b_valid_o  (b_valid),
      .axil_b_ready_i  (b_ready),
      .axil_ar_addr_i  (),
      .axil_ar_valid_i (ar_valid),
      .axil_ar_ready_o (ar_ready),
      .axil_ar_prot_i  (),
      .axil_r_data_o   (),
      .axil_r_resp_o   (),
      .axil_r_valid_o  (r_valid),
      .axil_r_ready_i  (r_ready),
      .ext_irq_i       ({1'h0, i2c_irqs}),
      .i2c_scl_pad_i   (i2c_scl_vip_dut),
      .i2c_scl_pad_o   (  /*NC*/),
      .i2c_scl_padoen_o(i2c_scl_dut_vip),
      .i2c_sda_padoen_o(i2c_sda_dut_vip),
      .i2c_sda_pad_o   (  /*NC*/),
      .i2c_sda_pad_i   (i2c_sda_vip_dut)
  );

endmodule : zeroheti_top_wrapper
