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

  longint unsigned cnt = 0;
  int unsigned     ps = 0;

  longint unsigned time_last = 0;
  longint unsigned period_last = 0;
  longint unsigned period_short = 0;
  longint unsigned period_long = 0;
  longint unsigned jitter_worst = 0;

  assign jitter_worst = period_long - period_short;

  always @(posedge clk_i) begin : prescaler
    if (ps >= prescaler_i) begin
      cnt++;
      ps = 0;
    end else ps++;
  end

  /*
  always @(cnt) begin : counter
  end
  */

  always @(posedge control_valid_i) begin

    if (time_last != 0) begin
      period_last = cnt - time_last;

      if (period_short == 0) begin
        period_short = period_last;
        period_long  = period_last;
      end else if (period_last < period_short) period_short = period_last;
      else if (period_last > period_long) period_long = period_last;

    end

    time_last = cnt;
  end

  always @(negedge enable_i) begin
    $display("[M%0d] worst jitter (cc): %0d", Idx, jitter_worst);
  end

endmodule : vip_motor_sim

