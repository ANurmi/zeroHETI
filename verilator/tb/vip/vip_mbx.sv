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

  localparam int unsigned MbxInboxAddr  = 32'h3_0000;
  localparam int unsigned MbxIrqAckAddr = 32'h3_0004;
  localparam int unsigned MbxTimeLoAddr = 32'h3_0008;
  localparam int unsigned MbxTimeHiAddr = 32'h3_000C;
  localparam int unsigned MbxM0StatAddr = 32'h3_0010;
  localparam int unsigned MbxM1StatAddr = 32'h3_0014;
  localparam int unsigned MbxM2StatAddr = 32'h3_0018;
  localparam int unsigned MbxM3StatAddr = 32'h3_001C;

  logic [31:0] last_seed = 0;

  initial begin
    irq_o   = 1'b0;
    rdata_o = 32'h0;
  end

  always @(posedge req_i) begin
    if (rst_ni) handle_tx();
  end

  task raise_irq();
    irq_o = 1;
  endtask

  task lower_irq();
    irq_o = 0;
  endtask

  task handle_tx();
    if (we_i) begin
      unique case (addr_i)
        MbxIrqAckAddr: begin
          zeroheti_top_wrapper.i_vip_i2c.i_ctrl_sim.ack_task(0);
          lower_irq();
        end
        MbxM0StatAddr: zeroheti_top_wrapper.i_vip_i2c.i_ctrl_sim.ack_task(5);
        MbxM1StatAddr: zeroheti_top_wrapper.i_vip_i2c.i_ctrl_sim.ack_task(6);
        MbxM2StatAddr: zeroheti_top_wrapper.i_vip_i2c.i_ctrl_sim.ack_task(7);
        MbxM3StatAddr: zeroheti_top_wrapper.i_vip_i2c.i_ctrl_sim.ack_task(8);
        default:;
      endcase
    end
    else begin
      if (addr_i == MbxInboxAddr) send_read();
    end
  endtask

  task send_read();
    last_seed = $urandom(last_seed);
    rdata_o   = last_seed;
  endtask


endmodule : vip_mbx

