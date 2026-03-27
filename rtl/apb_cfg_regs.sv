module apb_cfg_regs #(
) (
    input logic clk_i,
    input logic rst_ni,
    APB.Slave apb_i
);

  // TODO: add more relevant platform information/configuration
  logic [31:0] short_hash;
  assign short_hash   = 32'h`GIT_HASH;

  assign apb_i.prdata = short_hash;
  assign apb_i.pready = apb_i.psel & apb_i.penable;

endmodule : apb_cfg_regs

