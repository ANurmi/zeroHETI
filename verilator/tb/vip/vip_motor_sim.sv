module vip_motor_sim #(
    parameter int unsigned Idx = 0
) (
    input int unsigned prescaler_i,
    input logic        clk_i,
    input logic        enable_i,

    input  logic       control_valid_i,
    input  logic [7:0] control_wdata_i,
    output logic [7:0] control_rdata_o

);

  localparam int unsigned RndSeed = (721 * Idx + 1) % 100;

  assign control_rdata_o = 8'h67 + 8'(Idx);


endmodule : vip_motor_sim

