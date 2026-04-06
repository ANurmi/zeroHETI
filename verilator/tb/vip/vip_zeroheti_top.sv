module vip_zeroheti_top #(
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        sda_i,
    output logic        sda_o,
    input  logic        scl_i,
    output logic        scl_o,
    output logic [ 3:0] i2c_irq_o,
    output logic [31:0] aw_addr_o,
    output logic [ 2:0] aw_prot_o,
    output logic        aw_valid_o,
    input  logic        aw_ready_i,
    output logic [31:0] ar_addr_o,
    output logic [ 2:0] ar_prot_o,
    output logic        ar_valid_o,
    input  logic        ar_ready_i,
    output logic [31:0] w_data_o,
    output logic [ 3:0] w_strb_o,
    output logic        w_valid_o,
    input  logic        w_ready_i,
    input  logic        b_valid_i,
    input  logic [ 1:0] b_resp_i,
    output logic        b_ready_o,
    input  logic [31:0] r_data_i,
    input  logic [ 1:0] r_resp_i,
    input  logic        r_valid_i,
    output logic        r_ready_o
);

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32),
      .AXI_DATA_WIDTH(32)
  ) drv_bus ();

  vip_i2c i_vip_i2c (
      .clk_i,
      .rst_ni,
      .scl_o,
      .scl_i,
      .sda_i,
      .sda_o,
      .irq_o()
  );

  vip_mbx_driver i_mbx_drv (
      .clk_i,
      .rst_ni,
      .axi_mgr(drv_bus)
  );

  assign aw_valid_o       = drv_bus.aw_valid;
  assign drv_bus.aw_ready = aw_ready_i;

  assign ar_valid_o       = drv_bus.ar_valid;
  assign drv_bus.ar_ready = ar_ready_i;

  assign w_valid_o        = drv_bus.w_valid;
  assign drv_bus.w_ready  = w_ready_i;

  assign drv_bus.b_valid  = b_valid_i;
  assign b_ready_o        = drv_bus.b_ready;

  assign drv_bus.r_valid  = r_valid_i;
  assign r_ready_o        = drv_bus.r_ready;

endmodule : vip_zeroheti_top
