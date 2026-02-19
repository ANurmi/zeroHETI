//`define SIM_ASSERTS

module vip_motor_sim #(
    parameter int unsigned Idx = 0,
    parameter int unsigned TimestepSize = 100
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        enable_i,
    input  logic [31:0] voltage_i,
    output logic [31:0] speed_o,
    output logic        irq_o
);

  localparam int Resistance   = 100;  // Ohm
  localparam int SpeedMax     = 35_000;  // RPM
  localparam int VoltageMax   = 400_000;  // mV

  // Raise interrupt when warning tolerance exceeded
  localparam int SpeedTolWrn = 500;
  localparam int SpeedTolErr = SpeedTolWrn * 2;

  longint timestep  = 0;
  int unsigned ps   = 0;
  int unsigned seed = 0;

  int speed_real = 0;
  int speed_last = 0;
  int speed_bias = 0;
  int speed_ideal = 0;
  int speed_delta = 0;
  int acceleration[4:0] = {0, 0, 0, 0, 0};
  int unsigned voltage, power, power_last, transient;

  assign voltage = voltage_i;  // mV
  assign power = ((voltage ** 2) / (1000 * Resistance));  // mW

  // Correlate acceleration to change in power
  assign acceleration[0] = power - power_last - transient;

  assign speed_delta = speed_ideal - speed_real;
  assign speed_o = speed_real;

  initial begin
    irq_o = 1'b0;
  end

  always @(posedge clk_i) begin : timing_process
    if (enable_i) begin
      if (ps == TimestepSize) begin
        ps = 0;
        timestep++;
      end else ps++;
    end
  end

  always @(timestep) begin : simulation_process

    seed = timestep[31:0] % 10;
    irq_o = 1'b0;

    if (speed_real > 0) begin
      // Model transient enviromental distruptions with 5% probability
      transient  = ($urandom(seed) % 20 == 0) ? ($random(seed) % 1000) : 0;

      // Model constant enviromental distruptions, update with 5% probability
      speed_bias = ($urandom(seed) % 20 == 0) ? ($random(seed) % 10) : speed_bias;

      // Raise interrupt if warning threshhold exceeded
      if (abs(speed_delta) > SpeedTolErr) begin
        if (!accelerating()) irq_o = 1'b1;
      end
    end

    speed_last = speed_real;
    power_last = power;

    acceleration[4] = acceleration[3];
    acceleration[3] = acceleration[2];
    acceleration[2] = acceleration[1];
    acceleration[1] = acceleration[0];

    speed_real = speed_last + speed_bias
      + 1*(acceleration[4] / 10)
      + 2*(acceleration[3] / 10)
      + 5*(acceleration[2] / 10)
      + 2*(acceleration[1] / 10)
      + 1*(acceleration[0] / 10)
    ;

    speed_ideal = power;
  end

  function int abs( int x );
    if (x > 0) return x;
    else return x * (-1);
  endfunction

  function bit accelerating ();
    return ((acceleration[0] != 0) |
            (acceleration[1] != 0) |
            (acceleration[2] != 0) |
            (acceleration[3] != 0)
           );
  endfunction

`ifdef SIM_ASSERTS
  always @(timestep) begin : assertions

    if (speed_real > SpeedMax) $fatal(1, "Maximum speed exceeded!");

    if (abs(speed_delta) > SpeedTolErr) begin
        if (!accelerating()) $fatal(1, "Speed tolerance exceeded!");
    end
  end
`endif

endmodule : vip_motor_sim

