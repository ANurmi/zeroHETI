module vip_mbx #(
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        req_i,
    input  logic        we_i,
    output logic        irq_o,
    input  logic [31:0] addr_i,
    input  logic [31:0] wdata_i,
    output logic [31:0] rdata_o
);

  initial begin
    irq_o   = 1'b0;
    rdata_o = 32'h0;
  end

  always @(posedge req_i) begin
    if (rst_ni) handle_tx();
  end

  task test();
    $display("Hello from MBX");
  endtask

  task raise_irq();
    irq_o = 1;
  endtask

  task lower_irq();
    irq_o = 0;
  endtask

  task handle_tx();
    if (we_i) begin
      if (addr_i == 32'h30004) lower_irq();
    end
    else send_read();
  endtask

  task send_read();
    rdata_o = $random();
    @(posedge clk_i);
    @(posedge clk_i);
    @(negedge clk_i);
    rdata_o = 32'h0;
  endtask


endmodule : vip_mbx

