module zeroheti_top #()(
  input  logic clk_i,
  input  logic rst_ni,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o
);

//zeroheti_dbg_wrapper #() i_debug (
zeroheti_core #() i_core (
  .clk_i,
  .rst_ni,
  .testmode_i (1'b0),
  .jtag_tck_i,
  .jtag_tms_i,
  .jtag_trst_ni,
  .jtag_td_i,
  .jtag_td_o
);

endmodule : zeroheti_top

