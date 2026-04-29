module vip_motor_sim #(
    parameter int unsigned Idx = 0
) (
    input int unsigned prescaler_i,
    input logic        clk_i,
    input logic        enable_i,

    input  logic [15:0] speed_target_i,
    input  logic [ 7:0] speed_tune_i,
    output logic [15:0] speed_real_o,

    output logic irq_o
);

  localparam int unsigned RndSeed = (721 * Idx + 1) % 100;


endmodule : vip_motor_sim

