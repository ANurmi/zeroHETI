module vip_sim_env #(
    parameter type i2c_req_t = logic,
    parameter type i2c_rsp_t = logic
) (
    input  logic     clk_i,
    input  logic     rst_ni,
    input  i2c_req_t i2c_req_i,
    output i2c_rsp_t i2c_rsp_o
);

  localparam int unsigned NrMotors = 4;

  localparam logic [31:0] SimStartAddr = 32'h0100_0000;
  localparam logic [31:0] SimEndAddr = 32'h0100_0001;
  localparam logic [31:0] SimPsAddr = 32'h0100_0002;
  localparam logic [31:0] SimLfAddr = 32'h0100_0003;
  localparam logic [31:0] SimSeedAddr = 32'h0100_0004;
  localparam logic [31:0] SimPerAddr = 32'h0100_0005;

  localparam logic [31:0] DlMbxAddr = 32'h0200_0000;
  localparam logic [31:0] DlUpdAddr = 32'h0200_0001;
  localparam logic [31:0] DlCtrlAddr = 32'h0200_0002;
  localparam logic [31:0] DlRepAddr = 32'h0200_0003;

  localparam logic [31:0] MbxAckAddr = 32'h0300_0000;

  localparam logic [31:0] Ctrl0AckAddr = 32'h0301_0000;
  localparam logic [31:0] Ctrl1AckAddr = 32'h0301_0001;
  localparam logic [31:0] Ctrl2AckAddr = 32'h0301_0002;
  localparam logic [31:0] Ctrl3AckAddr = 32'h0301_0003;

  localparam logic [31:0] Rep0AckAddr = 32'h0401_0000;
  localparam logic [31:0] Rep1AckAddr = 32'h0401_0001;
  localparam logic [31:0] Rep2AckAddr = 32'h0401_0002;
  localparam logic [31:0] Rep3AckAddr = 32'h0401_0003;

  localparam logic [31:0] Upd0AckAddr = 32'h0501_0000;
  localparam logic [31:0] Upd1AckAddr = 32'h0501_0001;
  localparam logic [31:0] Upd2AckAddr = 32'h0501_0002;
  localparam logic [31:0] Upd3AckAddr = 32'h0501_0003;

  logic        [3:0] motor_irqs;
  logic              motor_enable;
  int unsigned       motor_prescaler;

  int unsigned       scb_loadfactor;
  int unsigned       scb_prescaler;
  int unsigned       scb_seed;
  logic              scb_enable;

  int unsigned       dl_mbx_us;
  int unsigned       dl_upd_us;
  int unsigned       dl_ctrl_us;
  int unsigned       dl_rep_us;

  typedef logic [31:0] dtype;
  typedef logic [6:0] atype;

  // use associative arrays for i2c memory space
  dtype array[atype];

  initial begin

    motor_prescaler = 0;
    scb_enable = 0;

    dl_mbx_us = 0;
    dl_upd_us = 0;
    dl_ctrl_us = 0;
    dl_rep_us = 0;

    array = '{
        7'h68 : 32'hBA11_55AB,
        7'h13 : 32'h0000_1234,
        7'h11 : 32'hb011_0c55,
        7'h0  : 32'hDEAD_BEEF
    };
  end

  logic sim_term_signal;
  assign sim_term_signal = i_zeroheti.i_core.dbg_bus.we
                         & i_zeroheti.i_core.dbg_bus.req
                         & i_zeroheti.i_core.dbg_bus.wdata[31]
                         & (i_zeroheti.i_core.dbg_bus.addr == 32'h0380);

  assign i2c_rsp_o.rdata = array[i2c_req_i.addr];

  always @(posedge i2c_req_i.valid) begin
    if (i2c_req_i.write) begin
      $display("[VIP_I2C] write - addr: %h, data: %h", i2c_req_i.addr, i2c_req_i.wdata);
    end else begin
      $display("[VIP_I2C] read  - addr: %h, data: %h", i2c_req_i.addr, array[i2c_req_i.addr]);
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
        .prescaler_i   (motor_prescaler),
        .enable_i      (motor_enable),
        .speed_target_i(),
        .speed_tune_i  (),
        .speed_real_o  (),
        .irq_o         (motor_irqs[i])
    );
  end

  vip_task_scoreboard i_scoreboard (
      .clk_i,
      .enable_i    (scb_enable),
      .prescaler_i (scb_prescaler),
      .loadfactor_i(scb_loadfactor),
      .seed_i      (scb_seed),
      .mbx_dl_us_i (dl_mbx_us),
      .upd_dl_us_i (dl_upd_us),
      .ctrl_dl_us_i(dl_ctrl_us),
      .rep_dl_us_i (dl_rep_us)
  );

  task automatic recv_letter(input logic [31:0] addr, input logic [31:0] data);
    unique case (addr)
      SimStartAddr: begin
        $display("[SCB] Simulation scoreboard active");
        motor_enable = 1'b1;
        scb_enable   = 1'b1;
      end
      SimEndAddr: begin
        $display("[SCB] Simulation complete");
        $display(" - Scoreboard task log: \n");
        motor_enable = 1'b0;
        scb_enable   = 1'b0;
      end
      SimLfAddr:   scb_loadfactor = data;
      SimPsAddr:   scb_prescaler = data;
      SimSeedAddr: scb_seed = data;
      //SimPerAddr: per_rep_us = data;
      DlMbxAddr:   dl_mbx_us = data;
      DlUpdAddr:   dl_upd_us = data;
      DlCtrlAddr:  dl_ctrl_us = data;
      DlRepAddr:   dl_rep_us = data;
      MbxAckAddr:  i_sim_env.i_scoreboard.retire_task(0);

      Ctrl0AckAddr: i_sim_env.i_scoreboard.retire_task(5);
      Ctrl1AckAddr: i_sim_env.i_scoreboard.retire_task(6);
      Ctrl2AckAddr: i_sim_env.i_scoreboard.retire_task(7);
      Ctrl3AckAddr: i_sim_env.i_scoreboard.retire_task(8);

      Upd0AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(1);
        else i_sim_env.i_scoreboard.activate_task(1);
      end

      Upd1AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(2);
        else i_sim_env.i_scoreboard.activate_task(2);
      end

      Upd2AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(3);
        else i_sim_env.i_scoreboard.activate_task(3);
      end

      Upd3AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(4);
        else i_sim_env.i_scoreboard.activate_task(4);
      end

      Rep0AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(9);
        else i_sim_env.i_scoreboard.activate_task(9);
      end
      Rep1AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(10);
        else i_sim_env.i_scoreboard.activate_task(10);
      end
      Rep2AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(11);
        else i_sim_env.i_scoreboard.activate_task(11);
      end
      Rep3AckAddr: begin
        if (data == 32'h0) i_sim_env.i_scoreboard.retire_task(12);
        else i_sim_env.i_scoreboard.activate_task(12);
      end

      default:
      $display("[VIP_SIM_ENV]: Warning! Received letter with unknown address: 0x%8h", addr);
    endcase
  endtask

endmodule : vip_sim_env
