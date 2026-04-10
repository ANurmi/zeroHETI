module vip_mbx_driver #(
    parameter type mbx_req_t = logic,
    parameter type mbx_rsp_t = logic
) (
    input logic           clk_i,
    input logic           rst_ni,
    input mbx_req_t       mbx_req_o,
    input mbx_rsp_t       mbx_rsp_i,
          AXI_LITE.Master axi_mgr
);
  // Assume similar address mapping from both sides of mailbox
  localparam logic [31:0] StatAddr = 32'h0003_0000;
  localparam logic [31:0] ObiCtrlAddr = 32'h0003_0004;
  localparam logic [31:0] AxiCtrlAddr = 32'h0003_0008;
  localparam logic [31:0] IboxAddrAddr = 32'h0003_000C;
  localparam logic [31:0] IboxDataAddr = 32'h0003_0010;
  localparam logic [31:0] OboxAddrAddr = 32'h0003_0014;
  localparam logic [31:0] OboxDataAddr = 32'h0003_001C;

  localparam time TA = 0.1ns;

  localparam int unsigned Prescaler = 'd1_000;
  int unsigned cnt = 0;

  always @(posedge clk_i) begin
    if (cnt >= Prescaler) begin
      poll_empty();
      cnt = 0;
    end else cnt++;
  end

  task automatic poll_empty();
    automatic logic [31:0] rdata;
    @(negedge clk_i);
    axi_mgr.ar_valid = 1;
    axi_mgr.ar_addr  = StatAddr;
    do begin
      @(negedge clk_i);
    end while (!axi_mgr.ar_ready);
    axi_mgr.ar_valid = 0;
    axi_mgr.r_ready  = 1;
    while (!axi_mgr.r_valid) begin
      @(negedge clk_i);
    end
    rdata = axi_mgr.r_data;
      @(negedge clk_i);
    axi_mgr.r_ready = 0;

    $display("rdata: %h", rdata);

  endtask


endmodule : vip_mbx_driver
