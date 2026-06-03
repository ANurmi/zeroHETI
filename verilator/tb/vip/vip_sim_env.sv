module vip_sim_env
  import rt_prof_pkg::*;
#(
    parameter type i2c_req_t = logic,
    parameter type i2c_rsp_t = logic
) (
    input  logic     clk_i,
    input  logic     rst_ni,
    input  i2c_req_t i2c_req_i,
    output i2c_rsp_t i2c_rsp_o
);

  localparam int unsigned NrMotors = 4;

  int unsigned             global_prescaler;

  logic                    motor_enable;
  logic        [3:0]       motor_control_valid;
  logic        [3:0][ 7:0] motor_control_wdata;
  logic        [3:0][ 7:0] motor_control_rdata;

  int unsigned             scb_loadfactor;
  int unsigned             scb_seed;
  logic                    scb_enable;

  int unsigned             dl_mbx_us;
  logic        [3:0][31:0] dl_upd_us;
  logic        [3:0][31:0] dl_ctrl_us;
  logic        [3:0][31:0] dl_rep_us;

  logic        [3:0][31:0] per_ctrl_us;

  initial begin

    global_prescaler = 0;
    scb_enable = 0;

    dl_mbx_us = 0;
    dl_upd_us = 0;
    dl_ctrl_us = 0;
    dl_rep_us = 0;
  end

  logic sim_term_signal;
  assign sim_term_signal = i_zeroheti.i_core.dbg_bus.we
                         & i_zeroheti.i_core.dbg_bus.req
                         & i_zeroheti.i_core.dbg_bus.wdata[31]
                         & (i_zeroheti.i_core.dbg_bus.addr == 32'h0380);

  assign i2c_rsp_o.rdata = (i2c_req_i.addr inside {[7'h10 : 7'h13]}) ?
      motor_control_rdata[i2c_req_i.addr[1:0]] : 0;

  always @(posedge i2c_req_i.valid) begin
    if (i2c_req_i.write) begin
      $display("[VIP_I2C] write - addr: %h, data: %h", i2c_req_i.addr, i2c_req_i.wdata);
      unique case (i2c_req_i.addr)
        7'h10: begin
          motor_control_wdata[0] = i2c_req_i.wdata[7:0];
          motor_control_valid[0] = 1'b1;
        end
        7'h11: begin
          motor_control_wdata[1] = i2c_req_i.wdata[7:0];
          motor_control_valid[1] = 1'b1;
        end
        7'h12: begin
          motor_control_wdata[2] = i2c_req_i.wdata[7:0];
          motor_control_valid[2] = 1'b1;
        end
        7'h13: begin
          motor_control_wdata[3] = i2c_req_i.wdata[7:0];
          motor_control_valid[3] = 1'b1;
        end
        default: $display("[VIP_I2C] warning! unknown i2c address");
      endcase
      @(posedge clk_i);
      motor_control_valid = 4'b0;
    end
  end

  always @(posedge sim_term_signal) begin
    // Clear out outbox when simulation is terminated.
    i_vip_zeroheti_top.i_mbx_drv.get_mail();
  end

  for (genvar i = 0; i < NrMotors; i++) begin : g_motors
    vip_motor_sim #(
        .Idx(i)
    ) i_motor (
        .clk_i,
        .prescaler_i    (global_prescaler),
        .enable_i       (motor_enable),
        .period_target_i(per_ctrl_us[i]),
        .control_valid_i(motor_control_valid[i]),
        .control_wdata_i(motor_control_wdata[i]),
        .control_rdata_o(motor_control_rdata[i])
    );
  end

  vip_task_scoreboard i_scoreboard (
      .clk_i,
      .enable_i    (scb_enable),
      .prescaler_i (global_prescaler),
      .loadfactor_i(scb_loadfactor),
      .seed_i      (scb_seed),
      .mbx_dl_us_i (dl_mbx_us),
      .upd_dl_us_i (dl_upd_us),
      .ctrl_dl_us_i(dl_ctrl_us),
      .rep_dl_us_i (dl_rep_us)
  );

  // Map mailbox letters to simulation environment
  task automatic recv_letter(input logic [31:0] addr, input logic [31:0] data);
    unique case (addr)
      SimMap.start: begin
        $display("[SCB] Simulation scoreboard active");
        motor_enable = 1'b1;
        scb_enable   = 1'b1;
      end
      SimMap.stop: begin
        $display("[SCB] Simulation complete");
        $display(" - Scoreboard task log: \n");
        motor_enable = 1'b0;
        scb_enable   = 1'b0;
      end
      SimMap.loadfactor: scb_loadfactor = data;
      SimMap.prescaler: global_prescaler = data;
      SimMap.seed: scb_seed = data;

      // Mbx task
      get_task_addrs(TASK_MBX).deadline: dl_mbx_us = data;
      // MBX_PER infered from LF
      get_task_addrs(TASK_MBX).ack: i_sim_env.i_scoreboard.retire_task(TASK_MBX);

      // Update tasks
      get_task_addrs(TASK_UPD_0).deadline: dl_upd_us[0] = data;
      get_task_addrs(TASK_UPD_1).deadline: dl_upd_us[1] = data;
      get_task_addrs(TASK_UPD_2).deadline: dl_upd_us[2] = data;
      get_task_addrs(TASK_UPD_3).deadline: dl_upd_us[3] = data;

      // UPD_PER not used
      get_task_addrs(
          TASK_UPD_0
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_UPD_0);
        else i_sim_env.i_scoreboard.activate_task(TASK_UPD_0);
      end

      get_task_addrs(
          TASK_UPD_1
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_UPD_1);
        else i_sim_env.i_scoreboard.activate_task(TASK_UPD_1);
      end

      get_task_addrs(
          TASK_UPD_2
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_UPD_2);
        else i_sim_env.i_scoreboard.activate_task(TASK_UPD_2);
      end

      get_task_addrs(
          TASK_UPD_3
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_UPD_3);
        else i_sim_env.i_scoreboard.activate_task(TASK_UPD_3);
      end
      // Control tasks
      get_task_addrs(TASK_CTRL_0).period: per_ctrl_us[0] = data;
      get_task_addrs(TASK_CTRL_1).period: per_ctrl_us[1] = data;
      get_task_addrs(TASK_CTRL_2).period: per_ctrl_us[2] = data;
      get_task_addrs(TASK_CTRL_3).period: per_ctrl_us[3] = data;

      get_task_addrs(TASK_CTRL_0).deadline: dl_ctrl_us[0] = data;
      get_task_addrs(TASK_CTRL_1).deadline: dl_ctrl_us[1] = data;
      get_task_addrs(TASK_CTRL_2).deadline: dl_ctrl_us[2] = data;
      get_task_addrs(TASK_CTRL_3).deadline: dl_ctrl_us[3] = data;

      get_task_addrs(TASK_CTRL_0).ack: i_sim_env.i_scoreboard.retire_task(TASK_CTRL_0);
      get_task_addrs(TASK_CTRL_1).ack: i_sim_env.i_scoreboard.retire_task(TASK_CTRL_1);
      get_task_addrs(TASK_CTRL_2).ack: i_sim_env.i_scoreboard.retire_task(TASK_CTRL_2);
      get_task_addrs(TASK_CTRL_3).ack: i_sim_env.i_scoreboard.retire_task(TASK_CTRL_3);

      // Report tasks
      get_task_addrs(TASK_REP_0).deadline: dl_rep_us[0] = data;
      get_task_addrs(TASK_REP_1).deadline: dl_rep_us[1] = data;
      get_task_addrs(TASK_REP_2).deadline: dl_rep_us[2] = data;
      get_task_addrs(TASK_REP_3).deadline: dl_rep_us[3] = data;

      get_task_addrs(
          TASK_REP_0
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_REP_0);
        else i_sim_env.i_scoreboard.activate_task(TASK_REP_0);
      end

      get_task_addrs(
          TASK_REP_1
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_REP_1);
        else i_sim_env.i_scoreboard.activate_task(TASK_REP_1);
      end

      get_task_addrs(
          TASK_REP_2
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_REP_2);
        else i_sim_env.i_scoreboard.activate_task(TASK_REP_2);
      end

      get_task_addrs(
          TASK_REP_3
      ).ack: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(TASK_REP_3);
        else i_sim_env.i_scoreboard.activate_task(TASK_REP_3);
      end
      default:
      $display("[VIP_SIM_ENV]: Warning! Received letter with unknown address: 0x%8h", addr);
    endcase
  endtask

endmodule : vip_sim_env
