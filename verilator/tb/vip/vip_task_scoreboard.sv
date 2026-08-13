module vip_task_scoreboard #(
) (
    input logic clk_i
);
  localparam int unsigned NrTasks = 3;


  // verilator lint_off UNOPTFLAT
  rt_prof_pkg::task_t ts[NrTasks];
  // verilator lint_on UNOPTFLAT
  longint unsigned g_counter = 0;



  always_ff @(posedge clk_i) begin : mbx_poll

    automatic bit mbx_empty;
    automatic rt_prof_pkg::letter_t letter;

    if (~scb_enable) begin
      i_vip.i_mbx_drv.is_empty(mbx_empty);

      while (~mbx_empty) begin
        i_vip.i_mbx_drv.get_letter(letter);
        read_letter(letter);
        i_vip.i_mbx_drv.is_empty(mbx_empty);
      end
    end

  end


  always_ff @(posedge clk_i) begin : global_counter
    g_counter += 1;
    for (int i = 0; i < NrTasks; i++) begin
      if (ts[i].available) ts[i].dl_cc += -1;
    end
  end


  for (genvar i = 0; i < NrTasks; i++) begin : g_retire
    always @(negedge ts[i].started) begin

      automatic int ret_last = ts[i].dl_cc;

      ts[i].available = 1'b0;

      if (ts[i].count_total == 0) begin
        ts[i].ret_worst_cc = ret_last;
        ts[i].ret_avg_cc   = ret_last;
      end else begin
        if (ret_last < ts[i].ret_worst_cc) begin
          ts[i].ret_worst_cc = ret_last;
        end
        ts[i].ret_avg_cc = ((ts[i].ret_avg_cc * ts[i].count_total) + ret_last) / (ts[i].count_total + 1);
      end

      if (ts[i].dl_cc < 0) begin
        ts[i].count_misses += 1;
      end

      ts[i].count_total += 1;

      ts[i].dl_cc = ts[i].dl_target_cc;
    end
  end



  // Hook into mmio regs in DUT
  bit scb_enable;
  assign scb_enable = i_dut.i_cfg_regs.gpreg_q[0][0];

  for (genvar i = 0; i < NrTasks; i++) begin : g_sim_hook
    assign ts[i].available = (ts[i].available) ? 1'b1:  i_dut.i_apb_timer.irq_o[(2*i)+1];
    assign ts[i].started   = i_dut.i_cfg_regs.gpreg_q[i+1][0];
  end


  always @(negedge scb_enable) begin
    $display("Task Scoreboard Log:");
    for (int i = 0; i < NrTasks; i++) begin
      $display("T%0d: total %3d, miss-%%:%3d, worst (cc): %5d, avg (cc): %5d", i,
               ts[i].count_total, (ts[i].count_misses * 100 / ts[i].count_total),
               ts[i].ret_worst_cc, ts[i].ret_avg_cc);
    end
  end

  task automatic read_letter(rt_prof_pkg::letter_t letter);
    // TODO: replace with address-derived indexing
    unique case (letter.addr)
      32'h1_0000: begin
        ts[0].dl_target_cc = letter.data;
        ts[0].dl_cc        = letter.data;
      end
      32'h1_0001: begin
        ts[1].dl_target_cc = letter.data;
        ts[1].dl_cc        = letter.data;
      end
      32'h1_0002: begin
        ts[2].dl_target_cc = letter.data;
        ts[2].dl_cc        = letter.data;
      end
      default: $display("Weird letter");
    endcase
  endtask

endmodule : vip_task_scoreboard

