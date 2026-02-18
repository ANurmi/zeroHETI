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

/* @electric_motors 1.12
* P = T × ω
* ω = 2 pi × f
* U = R × I
* P = U × I
* T = dL / dt (Newton II for rotation)
* K = pi × M × r²
* L = Kf
* f = P / K(df / dt) × 2pi
* f = f_new
* df/dt = (f_new - f_old) / timestep
* 1 Hz = 60 RPM
* */

  localparam int Resistance = 100; // Ohm
  localparam int PsLimit    = 100;
  localparam int SpeedMax   = 35_000; // RPM
  localparam int VoltageMax = 400_000; // mV
  localparam int unsigned K_inertia = 100;

  longint timestep = 0;
  int unsigned ps  = 0;

  int speed_real   = 0;
  int speed_last   = 0;
  int acceleration [4:0] = {0,0,0,0,0};
  int unsigned voltage, power, power_last, transient;

  assign voltage = voltage_i; // mV
  assign power   = ((voltage**2) / (1000*Resistance)); // mW


  // Correlate acceleration to change in power
  assign acceleration[0] = power - power_last - transient;

  assign speed_o = speed_real;

  initial begin
    irq_o = 1'b0;
  end

  always @(posedge clk_i) begin : timing_process
    if (enable_i) begin
      if (ps == PsLimit) begin
        ps = 0;
        timestep++;
      end else ps++;
    end
  end

  always @(timestep) begin : simulation_process
  
    if (speed_real > 0) begin
      // Model transient enviromental distruptions with 5% probability
      transient = ($urandom() % 20 == 0) ? ($urandom() % 1000) : 0;
    end
    speed_last = speed_real;
    power_last = power;

    acceleration[4] = acceleration[3];
    acceleration[3] = acceleration[2];
    acceleration[2] = acceleration[1];
    acceleration[1] = acceleration[0];

    speed_real = speed_last + 
      + 10*(acceleration[4] / 100)
      + 20*(acceleration[3] / 100)
      + 50*(acceleration[2] / 100)
      + 20*(acceleration[1] / 100)
      + 10*(acceleration[0] / 100)
    ;
  end

endmodule : vip_motor_sim

