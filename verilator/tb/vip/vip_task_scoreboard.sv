module vip_task_scoreboard #(
) (
    input logic            clk_i,
    input logic            enable_i,
    input longint unsigned mbx_dl_us_i,
    input longint unsigned wrn_dl_us_i,
    input longint unsigned rep_dl_us_i
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
    logic [63:0] dl_us;
    name_e       name;
  } task_t;

  task_t [TaskSetSize-1:0] task_set;

  initial begin

    @(posedge enable_i);

    for (int i = 0; i < TaskSetSize; i++) begin
      task_set[i].active = 1'b0;
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

  end

endmodule : vip_task_scoreboard

