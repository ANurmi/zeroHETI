module vip_task_scoreboard #(
) (
    input logic clk_i
);
  localparam int unsigned MicroNrTasks = 3;
  localparam int unsigned FullNrTasks = 5;

  localparam int unsigned MicroTaskWidth = $clog2(MicroNrTasks);
  localparam int unsigned FullTaskWidth = $clog2(FullNrTasks);

  // verilator lint_off UNOPTFLAT
  rt_prof_pkg::task_t ts_micro[MicroNrTasks];
  rt_prof_pkg::task_t ts_full[FullNrTasks];
  // verilator lint_on UNOPTFLAT

  longint unsigned g_counter = 0;

  always_ff @(posedge clk_i) begin : mbx_poll
    automatic bit mbx_empty;
    automatic rt_prof_pkg::letter_t letter;

    if (~scb_enable) begin
      i_vip.mbx_drv.is_empty(mbx_empty);
      while (~mbx_empty) begin
        i_vip.mbx_drv.get_letter(letter);
        read_letter(letter);
        i_vip.mbx_drv.is_empty(mbx_empty);
      end
    end
  end

  always_ff @(posedge clk_i) begin : global_counter
    g_counter += 1;

    for (int i = 0; i < MicroNrTasks; i++) begin : micro_decr
      if (ts_micro[i].available) ts_micro[i].dl_cc += -1;
    end

    for (int i = 0; i < FullNrTasks; i++) begin : full_decr
      if (ts_full[i].available) ts_full[i].dl_cc += -1;
    end
  end

  for (genvar i = 0; i < MicroNrTasks; i++) begin : g_retire_micro
    always @(negedge ts_micro[i].started) begin
      automatic int ret_last = ts_micro[i].dl_cc;
      ts_micro[i].available = 1'b0;

      if (ts_micro[i].count_total == 0) begin
        ts_micro[i].ret_worst_cc = ret_last;
        ts_micro[i].ret_avg_cc   = ret_last;
      end else begin
        if (ret_last < ts_micro[i].ret_worst_cc) ts_micro[i].ret_worst_cc = ret_last;
        ts_micro[i].ret_avg_cc = ((ts_micro[i].ret_avg_cc * ts_micro[i].count_total) + ret_last)
          / (ts_micro[i].count_total + 1);
      end

      if (ts_micro[i].dl_cc < 0) ts_micro[i].count_misses += 1;
      ts_micro[i].count_total += 1;
      ts_micro[i].dl_cc = ts_micro[i].dl_target_cc;
    end
  end

  for (genvar i = 0; i < FullNrTasks; i++) begin : g_retire_full
    always @(negedge ts_full[i].started) begin
      automatic int ret_last = ts_full[i].dl_cc;
      ts_full[i].available = 1'b0;

      if (ts_full[i].count_total == 0) begin
        ts_full[i].ret_worst_cc = ret_last;
        ts_full[i].ret_avg_cc   = ret_last;
      end else begin
        if (ret_last < ts_full[i].ret_worst_cc) ts_full[i].ret_worst_cc = ret_last;
        ts_full[i].ret_avg_cc = ((ts_full[i].ret_avg_cc * ts_full[i].count_total) + ret_last)
          / (ts_full[i].count_total + 1);
      end

      if (ts_full[i].dl_cc < 0) ts_full[i].count_misses += 1;
      ts_full[i].count_total += 1;
      ts_full[i].dl_cc = ts_full[i].dl_target_cc;
    end
  end

  bit scb_enable;
  bit micro_enable;
  bit rtprof_enable;

  // Hook into mmio regs in DUT
  assign micro_enable  = i_dut.i_cfg_regs.gpreg_q[0][0];
  assign rtprof_enable = i_dut.i_cfg_regs.gpreg_q[0][1];
  assign scb_enable    = micro_enable | rtprof_enable;

  for (genvar i = 0; i < MicroNrTasks; i++) begin : g_sim_hook_micro
    assign ts_micro[i].available = (ts_micro[i].available) ? 1'b1: i_dut.i_apb_timer.irq_o[(2*i)+1];
    assign ts_micro[i].started = i_dut.i_cfg_regs.gpreg_q[i+1][0];
  end
  for (genvar i = 0; i < FullNrTasks; i++) begin : g_sim_hook_full
    assign ts_full[i].available = (ts_full[i].available) ? 1'b1 : i_dut.i_apb_timer.irq_o[(2*i)+1];
    assign ts_full[i].started   = i_dut.i_cfg_regs.gpreg_q[i+1][0];
  end

  // Drive I2C VIP
  localparam logic [5:0][7:0] TestWord = 48'hDEADBEEFB00B;
  int unsigned test_idx = 0;
  int unsigned wr_count = 0;

  always @(negedge i_vip.i2c_0.tx_state.active) wr_count = 0;

  always @(negedge i_vip.i2c_0.tx_state.byte_active) begin : i2c_read
    if (i_vip.i2c_0.tx_state.addr_valid) begin
      if (~i_vip.i2c_0.we()) begin : read
        i_vip.i2c_0.set_rdata(TestWord[test_idx]);
        if (test_idx == 6) test_idx = 0;
        else test_idx++;
      end : read
      else begin : write
        if (wr_count > 32'h0) $display("[i2c]: %h", i_vip.i2c_0.wdata());
        wr_count++;
      end : write
    end
  end

  always @(negedge micro_enable) begin
    $display("[micro-rtprof] Task Scoreboard Log:");
    for (int i = 0; i < MicroNrTasks; i++) begin
      $display("T%0d: total %5d, miss-%%:%3d, worst (cc): %5d, avg (cc): %5d", i,
               ts_micro[i].count_total, (ts_micro[i].count_misses * 100 / ts_micro[i].count_total),
               ts_micro[i].ret_worst_cc, ts_micro[i].ret_avg_cc);
    end
  end
  always @(negedge rtprof_enable) begin
    $display("[rtprof] Task Scoreboard Log:");
    for (int i = 0; i < FullNrTasks; i++) begin
      $display("T%0d: total %5d, miss-%%:%3d, worst (cc): %5d, avg (cc): %5d", i,
               ts_full[i].count_total, (ts_full[i].count_misses * 100 / ts_full[i].count_total),
               ts_full[i].ret_worst_cc, ts_full[i].ret_avg_cc);
    end
  end

  task automatic read_letter(rt_prof_pkg::letter_t letter);

    unique case (letter.addr) inside
      [32'h1_0000 : 32'h1_1000]: begin
        ts_micro[MicroTaskWidth'(letter.addr)].dl_target_cc = letter.data;
        ts_micro[MicroTaskWidth'(letter.addr)].dl_cc        = letter.data;
      end

      [32'h2_0000 : 32'h2_1000]: begin
        ts_full[FullTaskWidth'(letter.addr)].dl_target_cc = letter.data;
        ts_full[FullTaskWidth'(letter.addr)].dl_cc        = letter.data;
      end

      default: ;
    endcase
  endtask

endmodule : vip_task_scoreboard

