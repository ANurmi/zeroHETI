module vip_task_scoreboard #(
) (
    input logic        clk_i,
    input logic        enable_i,
    input int unsigned prescaler_i,
    input int unsigned loadfactor_i,
    input int unsigned seed_i,
    input int unsigned mbx_dl_us_i,
    input int unsigned wrn_dl_us_i,
    input int unsigned rep_dl_us_i
);

  localparam int unsigned TaskSetSize = 9;

  typedef enum logic [1:0] {
    NONE = 0,
    MBX  = 1,
    WRN  = 2,
    REP  = 3
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


  task_t [TaskSetSize-1:0] task_set;
  task_ret_t [TaskSetSize-1:0] task_set_ret;

  longint unsigned counter_us = 0;
  int unsigned pre_counter = 0;

  always @(posedge clk_i) begin : us_counter
    if (enable_i) begin
      if (pre_counter == prescaler_i - 1) begin
        pre_counter = 0;
        counter_us++;
      end else pre_counter++;
    end
  end : us_counter

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

    // Produce randomized asynchronous events
    // - Load factor   0: ~1 / 1000 us
    // - Load factor 100: ~1 / 10 us
    if ($urandom_range(0, 999) < loadfactor_i + 1) begin
      // Issue mailbox task with 10 % probability
      if ($urandom_range(0, 9) < 1) begin
        // TODO:send letter
        //i_vip_zeroheti_top.i_mbx_drv.raise_irq();
        i_mbx_drv.raise_irq();
        activate_task(0);
      end
    end
  end : scb_main_proc

  initial begin

    @(posedge enable_i);

    // Seed random generator
    $urandom(seed_i);
    $urandom_range(seed_i);

    for (int i = 0; i < TaskSetSize; i++) begin
      task_set[i].active = 1'b0;
      task_set_ret[i] = '{default: 0};
      unique case (i) inside
        0: begin
          task_set[i].name  = MBX;
          task_set[i].dl_us = mbx_dl_us_i;
        end
        [1 : 4]: begin
          task_set[i].name  = WRN;
          task_set[i].dl_us = wrn_dl_us_i;
        end
        [5 : 8]: begin
          task_set[i].name  = REP;
          task_set[i].dl_us = rep_dl_us_i;
        end
      endcase
    end

    @(negedge enable_i);
    for (int i = 0; i < TaskSetSize; i++) begin
      $display("Task %0d - count: %3d, avg. slack: %4d us, worst slack: %4d us", i,
               task_set_ret[i].count, task_set_ret[i].slack_avg, task_set_ret[i].slack_worst);
    end

  end

  task automatic activate_task(input int idx);
    if (!task_set[idx].active) task_set[idx].active = 1;
  endtask

  task automatic retire_task(input int idx);

    if (task_set[idx].active) task_set[idx].active = 0;

    log_slack(idx);

    unique case (idx) inside
      0: task_set[idx].dl_us = mbx_dl_us_i;
      [1 : 4]: task_set[idx].dl_us = wrn_dl_us_i;
      [5 : 8]: task_set[idx].dl_us = rep_dl_us_i;
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


endmodule : vip_task_scoreboard

