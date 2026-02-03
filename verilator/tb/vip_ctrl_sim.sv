module vip_ctrl_sim #(
) (
    input logic clk_i
);

task write(input logic [6:0] addr, input logic [31:0] data);
  $display("[CTRL_SIM] write - addr: %h, data: %h", addr, data);
endtask

task read(input logic [6:0] addr, output logic [31:0] data);
  data = $random();
  $display("[CTRL_SIM] read - addr: %h, data: %h", addr, data);
endtask

endmodule : vip_ctrl_sim

