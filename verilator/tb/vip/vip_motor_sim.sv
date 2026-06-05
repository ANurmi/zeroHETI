module vip_motor_sim #(
    parameter int unsigned Idx = 0
) (
    input  int unsigned        prescaler_i,
    input  logic               clk_i,
    input  logic               enable_i,
    input  logic        [31:0] period_target_i,
    input  logic               control_valid_i,
    input  logic        [ 7:0] control_wdata_i,
    output logic        [ 7:0] control_rdata_o

);

  longint unsigned time_us = 0;
  longint unsigned time_last_us = 0;
  longint unsigned period_last_us = 0;
  longint unsigned period_long_us = 0;
  longint unsigned period_short_us = 0;
  longint unsigned jitter_worst_us = 0;

  int unsigned     ps = 0;

  assign jitter_worst_us = period_long_us - period_short_us;

  always @(posedge clk_i) begin : prescaler
    if (enable_i) begin
      if (ps >= prescaler_i) begin
        time_us++;
        ps = 0;
      end else ps++;
    end
  end

  // Drive control_rdata_o with randomized data
  initial begin
    // Define 127 as target setpoint for motor control
    control_rdata_o = 137;
  end

  always @(negedge control_valid_i) control_rdata_o = 8'($urandom_range(192, 64));


  // Reset counter whenever target period is adjusted
  always @(period_target_i) begin
    if (enable_i) begin
      time_last_us    = 0;
      period_last_us  = 0;
      period_long_us  = 0;
      period_short_us = 0;
    end
  end

  // Measure jitter from clock pulse when motor is serviced by write.
  always @(posedge control_valid_i) begin : jitter_update

    if (time_last_us != 0) begin
      period_last_us = time_us - time_last_us;
    end

    if (period_short_us == 0) begin
      period_long_us  = period_last_us;
      period_short_us = period_last_us;
    end else begin
      if (period_last_us < period_short_us) period_short_us = period_last_us;
      if (period_last_us > period_long_us) period_long_us = period_last_us;
    end

    time_last_us = time_us;

  end : jitter_update

  always @(negedge enable_i) begin
    $display("[M%0d] worst jitter (us): %0d", Idx, jitter_worst_us);
  end

endmodule : vip_motor_sim

