module zeroheti_top #()(
  input  logic clk_i,
  input  logic rst_ni,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o
);

zeroheti_dbg_wrapper #() i_debug (
  .clk_i,
  .rst_ni
);

endmodule : zeroheti_top

