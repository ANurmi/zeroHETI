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

  logic enable;

  always @(posedge enable) begin
    $display("[CTRL_SIM] Starting simulation");
    $display("[CTRL_SIM] Timing parameters: XXX");
    i_zeroheti.i_mbx.i_sim_mbx.raise_irq();
  end

  for (genvar i = 0; i < NrMotors; i++) begin : g_motors
    vip_motor_sim #(
        .Idx(i)
    ) i_motor (
        .clk_i,
        .rst_ni,
        .enable_i(enable),
        .irq_o   (irq_o[i])
    );
  end

  task write(input logic [6:0] addr, input logic [31:0] data);
    $display("[CTRL_SIM] write - addr: %h, data: %h", addr, data);
    unique case (addr)
      7'd0: enable = data[0];
      default: ;
    endcase
  endtask

  task read(input logic [6:0] addr, output logic [31:0] data);
    data = $random();
    $display("[CTRL_SIM] read - addr: %h, data: %h", addr, data);
  endtask

endmodule : vip_ctrl_sim

