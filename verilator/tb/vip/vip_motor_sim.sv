`define SIM_ASSERTS

module vip_motor_sim #(
    parameter int unsigned Idx = 0,
    parameter int unsigned TimestepSize = 100
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        enable_i,
    input  logic [31:0] voltage_i,
    input  logic [15:0] tune_i,
    input  logic        tune_vld_i,
    output logic [31:0] speed_o,
    output logic        irq_o
);

  localparam int R_motor = 10_000; // mOhm
  localparam int MtrTol  = 1_000; // RPM

  longint timestep  = 0;
  int unsigned ps   = 0;
  int seed = 721 * Idx;

  int voltage_real;
  int power_real, power_ideal;
  int speed_real, speed_ideal, delta;
  int env_lin, env_trans;

  always @(posedge clk_i) begin : timing_process
    if (enable_i) begin
      if (ps == TimestepSize) begin
        ps = 0;
        timestep++;
      end else ps++;
    end
  end

  initial begin
    env_lin   = 0;
    env_trans = 0;
    // Seed random generators
    $random(seed);
    $urandom(seed);
  end

  always @(timestep) begin : simulation_process

    voltage_real = voltage_i + 32'($signed(tune_i));

    power_real  = (voltage_real**2)/R_motor;
    power_ideal = (   voltage_i**2)/R_motor;

 //   seed = (Idx+700)*seed;

    if (speed_real > 0) begin
      // Model transient enviromental distruptions with 1% probability
      env_trans  = ($urandom() % 100 == 0) ? ($random() % 1200) : 0;

      // Model liner enviromental effects with changing direction
      env_lin = ($urandom() % 50 == 0) ? ($random() % 20) : env_lin;
    end

    // Directly correllate speed to power
    speed_ideal = power_ideal;
    speed_real  = power_real + env_lin + env_trans;
    speed_o     = speed_real;

    delta       = speed_ideal - speed_real;

    irq_o       = (abs(delta) > MtrTol) | irq_o;

  end

  // Writing tune value clears irq
  always @(posedge tune_vld_i) begin
    irq_o = 1'b0;
  end


  function int abs( int x );
    if (x > 0) return x;
    else return x * (-1);
  endfunction

endmodule : vip_motor_sim

