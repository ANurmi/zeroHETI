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

  longint unsigned            time_us = 0;
  longint unsigned            time_last_us = 0;
  longint unsigned            period_last_us = 0;
  longint unsigned            period_long_us = 0;
  longint unsigned            period_short_us = 0;
  longint unsigned            jitter_worst_us = 0;

  logic            [3:0][7:0] motor_state;
  logic            [7:0]      motor_state_avg;
  logic            [1:0]      state_ptr = 0;

  int unsigned                ps = 0;

  assign motor_state_avg = 8'((
    32'(motor_state[3]) +
    32'(motor_state[2]) +
    32'(motor_state[1]) +
    32'(motor_state[0])) /4 );


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
    for (int i = 0; i < 4; i++) begin
      motor_state[i] = 137;
    end
    // Define 127 as target setpoint for motor control
    control_rdata_o = 137;
  end

  always @(negedge control_valid_i) begin
    control_rdata_o        = motor_state_avg;
    motor_state[state_ptr] = control_wdata_i - 8'($urandom_range(16, 0));
    state_ptr++;
  end

  // Reset counter whenever target period is adjusted
  always @(period_target_i) begin
    if (enable_i) begin
      //time_last_us    = 0;
      time_last_us = time_us;
      period_last_us = 0;
      period_long_us = 0;
      period_short_us = 0;
    end
  end

  // Measure jitter from clock pulse when motor is serviced by write.
  always @(posedge control_valid_i) begin : jitter_update

    if (time_us != 0) begin
      period_last_us = time_us - time_last_us;
    end

    if (period_short_us == 0) begin
      period_long_us  = period_last_us;
      period_short_us = period_last_us;
    end else begin
      if (period_last_us < period_short_us) period_short_us = period_last_us;
      if (period_last_us > period_long_us) period_long_us = period_last_us;
      // Update jitter only for non-zero long+short period
      if (period_long_us != 0) jitter_worst_us = period_long_us - period_short_us;
    end

    time_last_us = time_us;

  end : jitter_update

  always @(negedge enable_i) begin
    $display("[M%0d] worst jitter (us): %0d", Idx, jitter_worst_us);
  end

endmodule : vip_motor_sim

