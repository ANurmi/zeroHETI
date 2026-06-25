module vip_task_scoreboard
  import rt_prof_pkg::*;
#(
) (
    input logic                    clk_i,
    input logic                    enable_i,
    input int unsigned             prescaler_i,
    input int unsigned             loadfactor_i,
    input int unsigned             seed_i,
    input int unsigned             mbx_dl_us_i,
    input logic        [3:0][31:0] upd_dl_us_i,
    input logic        [3:0][31:0] ctrl_dl_us_i,
    input logic        [3:0][31:0] rep_dl_us_i
);

  task_t           [TaskSetSize-1:0] task_set;
  task_ret_t       [TaskSetSize-1:0] task_set_ret;

  longint unsigned                   counter_us = 0;
  int unsigned                       pre_counter = 0;
  int unsigned                       mbx_task_per;

  assign mbx_task_per = 'd8000;  // needs to be sparse enough

  always @(posedge clk_i) begin : us_counter
    if (enable_i) begin
      if (pre_counter >= prescaler_i - 1) begin
        pre_counter = 0;
        counter_us++;
      end else pre_counter++;
    end
  end : us_counter

  always @(i_zeroheti.i_apb_timer.irq_o) begin
    if (enable_i) begin
      if (i_zeroheti.i_apb_timer.irq_o[1]) activate_task(TASK_CTRL_0);
      if (i_zeroheti.i_apb_timer.irq_o[3]) activate_task(TASK_CTRL_1);
      if (i_zeroheti.i_apb_timer.irq_o[5]) activate_task(TASK_CTRL_2);
      if (i_zeroheti.i_apb_timer.irq_o[7]) activate_task(TASK_CTRL_3);
    end
  end

  always @(counter_us) begin : scb_main_proc

    // Check for deadline misses
    for (int i = 0; i < TaskSetSize; i++) begin
      if (task_set[i].active & task_set[i].dl_us == 0) begin
        $fatal(1, "Deadline miss for task %0d!", i);
      end
    end

    // Decrement DL of active tasks
    for (int i = 0; i < TaskSetSize; i++) begin
      if (task_set[i].active) task_set[i].dl_us--;
    end

  end : scb_main_proc

  always @(counter_us) begin : scb_mbx_proc
    // Activate mailbox task periodically
    if ((counter_us % 64'(mbx_task_per) == 0) | counter_us == 10) begin
      activate_task(TASK_MBX);
      i_mbx_drv.send_letter(32'h100, generate_directive());
      i_mbx_drv.send_letter(32'h101, generate_directive());
      i_mbx_drv.send_letter(32'h102, generate_directive());
      i_mbx_drv.send_letter(32'h103, generate_directive());
      i_mbx_drv.raise_irq();
    end
  end : scb_mbx_proc


  initial begin

    @(posedge enable_i);

    // Seed random generator
    $urandom(seed_i);
    $urandom_range(seed_i);

    for (int i = 0; i < TaskSetSize; i++) begin

      task_set[i].active = 1'b0;
      task_set_ret[i] = '{default: 0};

      unique case (i) inside
        TASK_MBX: begin
          task_set[i].name  = TASK_MBX;
          task_set[i].dl_us = mbx_dl_us_i;
        end
        [TASK_UPD_0 : TASK_UPD_3]: begin
          task_set[i].name  = task_num_e'(i);
          task_set[i].dl_us = upd_dl_us_i[i-1];
        end
        [TASK_CTRL_0 : TASK_CTRL_3]: begin
          task_set[i].name  = task_num_e'(i);
          task_set[i].dl_us = ctrl_dl_us_i[i-5];
        end
        [TASK_REP_0 : TASK_REP_3]: begin
          task_set[i].name  = task_num_e'(i);
          task_set[i].dl_us = rep_dl_us_i[i-9];
        end
      endcase
    end

    @(negedge enable_i);
    for (int i = 0; i < TaskSetSize; i++) begin
      automatic string TaskName;
      unique case (i) inside
        TASK_MBX:                    TaskName = "GetMail";
        [TASK_UPD_0 : TASK_UPD_3]:   TaskName = {"UpdateCtrl", string'(i - 1 + 48)};
        [TASK_CTRL_0 : TASK_CTRL_3]: TaskName = {"I2cCtrl", string'(i - 5 + 48)};
        [TASK_REP_0 : TASK_REP_3]:   TaskName = {"I2cReport", string'(i - 9 + 48)};
      endcase
      $display("T%02d ( %11s ) - count: %3d, avg. slack: %4d us, worst slack: %4d us", i, TaskName,
               task_set_ret[i].count, task_set_ret[i].slack_avg, task_set_ret[i].slack_worst);
    end
    $display("");

  end

  task automatic activate_task(input int idx);
    if (!task_set[idx].active) begin
      task_set[idx].active = 1;
    end else begin
      if (enable_i) begin
        $display("[Warning] re-pended active task [%2d] at time %0d", idx, counter_us);
      end
    end
  endtask

  task automatic retire_task(input int idx);

    if (task_set[idx].active) task_set[idx].active = 0;

    log_slack(idx);

    unique case (idx) inside
      TASK_MBX:                    task_set[idx].dl_us = mbx_dl_us_i;
      [TASK_UPD_0 : TASK_UPD_3]:   task_set[idx].dl_us = upd_dl_us_i[idx-1];
      [TASK_CTRL_0 : TASK_CTRL_3]: task_set[idx].dl_us = ctrl_dl_us_i[idx-5];
      [TASK_REP_0 : TASK_REP_3]:   task_set[idx].dl_us = rep_dl_us_i[idx-9];
    endcase

  endtask

  task automatic log_slack(input int unsigned i);
    if (task_set_ret[i].count == 0) begin  // initial state
      task_set_ret[i].slack_worst = task_set[i].dl_us;
      task_set_ret[i].slack_avg   = task_set[i].dl_us;
    end else begin
      // update worst slack
      if (task_set_ret[i].slack_worst > task_set[i].dl_us) begin
        task_set_ret[i].slack_worst = task_set[i].dl_us;
      end

      // update average slack
      task_set_ret[i].slack_avg = ((task_set_ret[i].slack_avg * task_set_ret[i].count)
            + task_set[i].dl_us) / (task_set_ret[i].count + 32'd1);
    end

    task_set_ret[i].count += 1;
  endtask

  function automatic logic [31:0] generate_directive();
    // WCET for 4*I2C sequential I2C operations: ~560 us
    automatic logic [31:0] MinPeriod = 'd900;
    automatic logic [31:0] directive;
    automatic int unsigned load = (loadfactor_i * $urandom_range(100, 0)) / 100;
    automatic logic [1:0] key = (load > 50) ? 0 : (load > 30) ? 1 : (load > 10) ? 2 : 3;
    unique case (key)
      2'd0: directive = MinPeriod * 1;
      2'd1: directive  = MinPeriod * 3;
      2'd2: directive  = MinPeriod * 5;
      2'd3: directive   = MinPeriod * 9;
      default: directive    = MinPeriod * 4;
    endcase
    return directive;
  endfunction

endmodule : vip_task_scoreboard

