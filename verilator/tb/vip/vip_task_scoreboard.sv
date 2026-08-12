module vip_task_scoreboard #(
) (
    input logic clk_i
);

  localparam int unsigned NrTasks = 4;

  typedef struct packed {
    bit              active;
    longint unsigned exec_cc;
  } task_t;

  task_t task_set[NrTasks];

  longint unsigned g_counter = 0;

  always_ff @(posedge clk_i) begin
    i_vip.i_mbx_drv.poll_empty();
  end

  always_ff @(posedge clk_i) begin

    g_counter += 1;

    for (int i = 0; i < NrTasks; i++) begin
      if (task_set[i].active) task_set[i].exec_cc += 1;
    end

  end

  for (genvar i = 0; i < NrTasks; i++) begin : g_sim_hook
    assign task_set[i].active = i_dut.i_cfg_regs.gpreg_q[i+1][0];
  end

  bit scb_enable;
  assign scb_enable = i_dut.i_cfg_regs.gpreg_q[0][0];

  final begin
    if (scb_enable) begin
      for (int i = 0; i < NrTasks; i++) begin
        $display("Task %0d cc %0d", i, task_set[i].exec_cc);
      end
    end
  end

endmodule : vip_task_scoreboard

