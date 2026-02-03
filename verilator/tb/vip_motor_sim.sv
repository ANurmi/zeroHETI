module vip_motor_sim #(
    parameter int unsigned Idx = 0
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic enable_i,
    output logic irq_o
);

  logic [63:0] local_cnt = 0;

  real S_target = 0;

  initial begin
    irq_o = 1'b0;
  end

  always @(enable_i) begin
    if (rst_ni) begin
      if (enable_i) $display("[M%0d] enabled", Idx);
      else $display("[M%0d] disabled", Idx);
    end
  end

  always @(posedge clk_i) begin
    if (enable_i) local_cnt++;
  end

endmodule : vip_motor_sim

