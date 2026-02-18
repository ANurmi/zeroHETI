module vip_motor_sim #(
    parameter int unsigned Idx = 0
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        enable_i,
    input  logic [31:0] voltage_i,
    output logic [31:0] speed_o,
    output logic        irq_o
);

  int speed_real   = 32'hBEBEFACE;
  int speed_target = 0;

  assign speed_o = speed_real;

  initial begin
    irq_o = 1'b0;
  end

endmodule : vip_motor_sim

