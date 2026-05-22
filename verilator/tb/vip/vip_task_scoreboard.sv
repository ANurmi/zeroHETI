module vip_task_scoreboard #(
) (
    input logic        clk_i,
    input logic        enable_i,
    input int unsigned prescaler_i,
    input int unsigned loadfactor_i,
    input int unsigned seed_i,
    input int unsigned mbx_dl_us_i,
    input int unsigned upd_dl_us_i,
    input int unsigned ctrl_dl_us_i,
    input int unsigned rep_dl_us_i
);

  localparam int unsigned PerTaskSetSize = 13;

  typedef enum logic [2:0] {
    NONE = 0,
    MAIL = 1,
    UPDATE = 2,
    CONTROL = 3,
    REPORT = 4
  } name_e;

  typedef struct packed {
    bit          active;
    logic [31:0] dl_us;
    name_e       name;
  } task_t;


  typedef struct packed {
    int unsigned count;
    int unsigned slack_worst;
    int unsigned slack_avg;
  } task_ret_t;


  task_t [PerTaskSetSize-1:0] task_set;
  task_ret_t [PerTaskSetSize-1:0] task_set_ret;

  longint unsigned counter_us = 0;
  int unsigned pre_counter = 0;
  int unsigned mbx_task_per;

  assign mbx_task_per = 3 * mbx_dl_us_i + (4 * (100 - loadfactor_i));

  always @(posedge clk_i) begin : us_counter
    if (enable_i) begin
      if (pre_counter == prescaler_i - 1) begin
        pre_counter = 0;
        counter_us++;
      end else pre_counter++;
    end
  end : us_counter

  always @(i_zeroheti.i_apb_timer.irq_o) begin
    if (enable_i) begin
      if (i_zeroheti.i_apb_timer.irq_o[1]) activate_task(5);
      if (i_zeroheti.i_apb_timer.irq_o[3]) activate_task(6);
      if (i_zeroheti.i_apb_timer.irq_o[5]) activate_task(7);
      if (i_zeroheti.i_apb_timer.irq_o[7]) activate_task(8);
    end
  end

  always @(counter_us) begin : scb_main_proc

    // Check for deadline misses
    for (int i = 0; i < PerTaskSetSize; i++) begin
      if (task_set[i].active & task_set[i].dl_us == 0) begin
        $fatal(1, "Deadline miss for task %0d!", i);
      end
    end

    // Decrement DL of active tasks
    for (int i = 0; i < PerTaskSetSize; i++) begin
      if (task_set[i].active) task_set[i].dl_us--;
    end

  end : scb_main_proc

  always @(counter_us) begin : scb_mbx_proc
    // Activate mailbox task externally periodically
    if (counter_us % 64'(mbx_task_per) == 0) begin
      activate_task(0);
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

    for (int i = 0; i < PerTaskSetSize; i++) begin
      task_set[i].active = 1'b0;
      task_set_ret[i] = '{default: 0};
      unique case (i) inside
        0: begin
          task_set[i].name  = MAIL;
          task_set[i].dl_us = mbx_dl_us_i;
        end
        default: ;
        [1 : 4]: begin
          task_set[i].name  = UPDATE;
          task_set[i].dl_us = upd_dl_us_i;
        end
        [5 : 8]: begin
          task_set[i].name  = CONTROL;
          task_set[i].dl_us = ctrl_dl_us_i;
        end
        [9 : 12]: begin
          task_set[i].name  = REPORT;
          task_set[i].dl_us = rep_dl_us_i;
        end
      endcase
    end

    @(negedge enable_i);
    for (int i = 0; i < PerTaskSetSize; i++) begin
      automatic string TaskName;
      unique case (i) inside
        0: TaskName = "GetMail";
        [1 : 4]: TaskName = {"UpdateCtrl", string'(i - 1 + 48)};
        [5 : 8]: TaskName = {"I2cCtrl", string'(i - 5 + 48)};
        [9 : 12]: TaskName = {"I2cReport", string'(i - 9 + 48)};
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
      $display("[Warning] re-pended active task");
    end
  endtask

  task automatic retire_task(input int idx);

    if (task_set[idx].active) task_set[idx].active = 0;

    log_slack(idx);

    unique case (idx) inside
      0: task_set[idx].dl_us = mbx_dl_us_i;
      [1 : 4]: task_set[idx].dl_us = upd_dl_us_i;
      [5 : 8]: task_set[idx].dl_us = ctrl_dl_us_i;
      [9 : 12]: task_set[idx].dl_us = rep_dl_us_i;
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
    return 32'hDEADBEEF;
  endfunction


endmodule : vip_task_scoreboard

