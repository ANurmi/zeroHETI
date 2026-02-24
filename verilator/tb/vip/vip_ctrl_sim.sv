module vip_ctrl_sim #(
    localparam int unsigned NrIrqs   = 4,
    localparam int unsigned NrMotors = NrIrqs
) (
    input  logic              clk_i,
    input  logic              rst_ni,
    output logic [NrIrqs-1:0] irq_o
);

  /* Internal address mapping 
* 0: 31'h0, sim_en
* 1: reserved
* 2: M0 status
* 3: M0 control
* 4: M1 status
* 5: M1 control
* 6: M2 status
* 7: M2 control
* 8: M3 status
* 9: M3 control
* */

  localparam int unsigned TaskSetSize = 9;

  localparam longint unsigned MbxPerUs = 'd7_000;
  localparam longint unsigned RepPerUs = 'd5_000;

  localparam longint unsigned RepOfsUs = 'd0_150;
  localparam longint unsigned MbxOfsUs = 'd2_000;

  localparam longint unsigned MbxDlUs = 'd4_000;
  localparam longint unsigned WrnDlUs = 'd1_000;
  localparam longint unsigned RepDlUs = 'd2_000;

  logic [3:0][31:0] voltages = 0;
  logic [3:0][15:0] tune     = 0;
  logic [3:0]       tune_vld = 0;
  logic [3:0][31:0] speeds   = 0;

  typedef enum logic [1:0] {
    MBX = 0,
    WRN = 1,
    REP = 2
  } name_e;

  typedef struct packed {
    name_e       name;
    bit          active;
    logic [63:0] dl_us;
  } task_t;

  task_t [TaskSetSize-1:0] task_set;

  initial begin
    for (int i = 0; i < TaskSetSize; i++) begin
      task_set[i].active = 1'b0;
      unique case (i) inside
        0: begin
          task_set[i].name  = MBX;
          task_set[i].dl_us = MbxDlUs;
        end
        [1 : 4]: begin
          task_set[i].name  = WRN;
          task_set[i].dl_us = WrnDlUs;
        end
        [5 : 8]: begin
          task_set[i].name  = REP;
          task_set[i].dl_us = RepDlUs;
        end
      endcase
    end
  end

  logic enable;

  logic [63:0] time_us = 0;
  logic [7:0] prescaler = 0;

  // Assume 10 MHz sim clock frequency
  always @(posedge clk_i) begin : g_us_counter
    if (prescaler == 'd9) begin
      prescaler = 0;
      time_us++;
    end else prescaler++;
  end


  always @(posedge enable) begin
    time_us = 0; // clear vip time when first enabled
    $display("[CTRL_SIM] Starting simulation for task set:");
    $display("[CTRL_SIM] (Periodic): Receive MBX directive,          DL: %3d us", MbxDlUs);
    $display("[CTRL_SIM] (Sporadic): Motor [0-3] speed warning,      DL: %3d us", WrnDlUs);
    $display("[CTRL_SIM] (Periodic): Report M[0-3] speed, timestamp, DL: %3d us", RepDlUs);
  end



  always @(time_us) begin : g_dl_counter

    tune_vld = 0;

    // Decrement deadlines of active task
    for (int i = 0; i < TaskSetSize; i++) begin
      if (task_set[i].active) begin
        if (task_set[i].dl_us == 0) $fatal(1, "Deadline miss for task idx %0d!", i);
        else task_set[i].dl_us--;
      end
    end

    // Activate periodic directive task
    if (enable & ((time_us - MbxOfsUs) % MbxPerUs == 0)) begin
      pend_task(0);
      reset_task_dl(0);
      i_zeroheti.i_mbx.i_sim_mbx.raise_irq();
    end

    // Activate periodic reporting tasks with appropriate offset
    for (int i=0; i<4; i++) begin
      if (enable & ((time_us - RepOfsUs*i) % RepPerUs == 0)) begin
        pend_task(5+i);
        reset_task_dl(5+i);
      end
    end

    // Dectivate warning tasks
    for (int i=0; i< NrMotors; i++) begin
      automatic int SporIdx = i + 1;
      if (~irq_o[i]) begin
        ack_task(SporIdx);
      end
    end

  end : g_dl_counter

  for (genvar i = 0; i < NrMotors; i++) begin : g_sporadic_irqs
    localparam int SporIdx = i + 1;
    always @(posedge irq_o[i]) begin
      if (enable) begin
        pend_task(SporIdx);
        reset_task_dl(SporIdx);
      end
    end
  end

  for (genvar i = 0; i < NrMotors; i++) begin : g_motors
    vip_motor_sim #(
        .Idx(i)
    ) i_motor (
        .clk_i,
        .rst_ni,
        .enable_i (enable),
        .voltage_i(voltages[i]),
        .tune_i   (tune[i]),
        .tune_vld_i (tune_vld[i]),
        .speed_o  (speeds[i]),
        .irq_o    (irq_o[i])
    );
  end

  task write(input logic [6:0] addr, input logic [31:0] data);
    automatic string addr_name = "";
    unique case (addr)
      7'd0: begin
        enable    = data[0];
        addr_name = "SimCtrl";
      end
      7'd2: begin 
        voltages[0] = data;
        tune[0]     = 0;
        addr_name   = "M0_Ctrl";
      end
      7'd3: begin
        tune[0] = data[15:0];
        tune_vld[0] = 1'b1;
        addr_name = "M0_Tune";
      end
      7'd5: begin
        voltages[1] = data; 
        tune[1]     = 0;
        addr_name   = "M1_Ctrl";
      end
      7'd6: begin
        tune[1] = data[15:0];
        tune_vld[1] = 1'b1;
        addr_name = "M1_Tune";
      end
      7'd8: begin
        voltages[2] = data;
        tune[2]     = 0;
        addr_name   = "M2_Ctrl";
      end
      7'd9: begin
        tune[2] = data[15:0];
        tune_vld[2] = 1'b1;
        addr_name = "M2_Tune";
      end
      7'd11: begin
        voltages[3] = data;
        tune[3]     = 0;
        addr_name   = "M3_Ctrl";
      end
      7'd12: begin
        tune[3] = data[15:0];
        tune_vld[3] = 1'b1;
        addr_name = "M3_Tune";
      end
      default: ;
    endcase
    $display("[CTRL_SIM] write - addr: %s, data: 0x%h", addr_name, data);
  endtask

  task read(input logic [6:0] addr, output logic [31:0] data);
    automatic string addr_name = "";
    unique case (addr)
      7'd1: begin
        data = speeds[0];
        addr_name = "M0_Stat";
      end
      7'd4: begin
        data = speeds[1];
        addr_name = "M1_Stat";
      end
      7'd7: begin
        data = speeds[2];
        addr_name = "M2_Stat";
      end
      7'd10: begin
        data = speeds[3];
        addr_name = "M3_Stat";
      end
      default: ;
    endcase
    $display("[CTRL_SIM] read  - addr: %s, data: 0x%h", addr_name, data);
  endtask

  task pend_task(input int unsigned i);
    task_set[i].active = 1'b1;
  endtask

  task ack_task(input int unsigned i);
    task_set[i].active = 1'b0;
  endtask

  task reset_task_dl(input int unsigned i);
    unique case (i) inside
      0:       task_set[i].dl_us = MbxDlUs;
      [1 : 4]: task_set[i].dl_us = WrnDlUs;
      [5 : 8]: task_set[i].dl_us = RepDlUs;
    endcase
  endtask

endmodule : vip_ctrl_sim

