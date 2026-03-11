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

  localparam int unsigned LoadFactor = 0;

// verilator lint_off WIDTHTRUNC
// verilator lint_off WIDTHEXPAND
  localparam longint R_motor = 10_000; // mOhm
  localparam int MtrTol  = 1_000; // RPM

  longint timestep  = 0;
  int unsigned ps   = 0;
  int seed = (721 * Idx + 1) % 100;

  longint voltage_real;
  longint power_real, power_ideal;

  longint speed_real, speed_ideal, delta;
  longint env_lin, env_trans;

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

    voltage_real = 64'(voltage_i) + 64'($signed(tune_i));

    power_real  = (voltage_real**2)/R_motor;
    power_ideal = (64'(voltage_i)**2)/R_motor;

    if (speed_real > 0) begin
      // Model transient enviromental distruptions with x% probability
      env_trans  = (64'($urandom()) % (120-LoadFactor) == 0) ? (64'($random()) % 1200) : 0;

      // Model linear enviromental effects with changing direction
      env_lin = (64'($urandom()) % 50 == 0) ? (64'($random()) % 20) : env_lin;
    end

    // Directly correllate speed to power
    speed_ideal = power_ideal;
    speed_real  = 32'(power_real + env_lin + env_trans);
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
// verilator lint_on WIDTHTRUNC
// verilator lint_on WIDTHEXPAND

endmodule : vip_motor_sim

