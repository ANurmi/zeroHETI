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
  localparam logic [31:0] OboxDataAddr = 32'h0003_0018;

  localparam time TA = 0.1ns;

  typedef struct packed {
    logic [31:0] addr;
    logic [31:0] data;
  } letter_t;

  localparam int unsigned Prescaler = 'd600;
  int unsigned cnt = 0;

  always @(posedge clk_i) begin
    if (cnt >= Prescaler) begin
      poll_empty();
      cnt = 0;
    end else cnt++;
  end

  task automatic axi_read(input logic [31:0] addr, output logic [31:0] rdata);
    @(negedge clk_i);
    axi_mgr.ar_valid = 1;
    axi_mgr.ar_addr  = addr;
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
  endtask

  task automatic axi_write(input logic [31:0] addr, input logic [31:0] wdata);
    automatic bit got_aw, got_w = 0;
    @(negedge clk_i);
    axi_mgr.aw_valid = 1;
    axi_mgr.aw_addr  = addr;
    axi_mgr.w_valid  = 1;
    axi_mgr.w_data   = wdata;
    axi_mgr.w_strb   = 4'hf;
    do begin
      @(negedge clk_i);
      if (axi_mgr.aw_ready) begin
        got_aw = 1;
        axi_mgr.aw_valid = 0;
        axi_mgr.aw_addr = 32'h0;
      end
      if (axi_mgr.w_ready) begin
        got_w = 1;
        axi_mgr.w_valid = 0;
        axi_mgr.w_data = 32'h0;
        axi_mgr.w_strb = 4'h0;
      end
    end while (!(got_aw & got_w));
    axi_mgr.b_ready = 1;
    while (!axi_mgr.b_valid) @(negedge clk_i);
    @(negedge clk_i);
    axi_mgr.b_ready = 0;
  endtask

  task automatic poll_empty();
    automatic logic [31:0] status;
    axi_read(StatAddr, status);
    if (~status[2]) get_mail();
  endtask

  task automatic get_mail();
    automatic letter_t letter;
    axi_read(OboxAddrAddr, letter.addr);
    axi_read(OboxDataAddr, letter.data);
    $display("[VIP_AXI] got letter, addr: %08h data: %08h", letter.addr, letter.data);
    // Ack letter after read
    axi_write(AxiCtrlAddr, 32'h1);
  endtask


endmodule : vip_mbx_driver
