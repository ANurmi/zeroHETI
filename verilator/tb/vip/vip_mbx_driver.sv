module vip_mbx_driver #(
) (
    input logic           clk_i,
    input logic           rst_ni,
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

  localparam logic [31:0] SimParam0 = 32'h0100_0000;
  localparam logic [31:0] SimParam1 = 32'h0200_0000;
  localparam logic [31:0] SimParam2 = 32'h0300_0000;
  localparam logic [31:0] SimParam3 = 32'h0400_0000;

  typedef struct packed {
    logic [31:0] addr;
    logic [31:0] data;
  } letter_t;

  int unsigned ps = 'd600;
  int unsigned cnt = 0;

  always @(posedge clk_i) begin
    if (cnt >= ps) begin
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
    while (~status[2]) begin
      get_mail();
      axi_read(StatAddr, status);
    end
  endtask

  task automatic get_mail();

    automatic letter_t letter;
    axi_read(OboxAddrAddr, letter.addr);
    axi_read(OboxDataAddr, letter.data);

    // Ack letter after read
    axi_write(AxiCtrlAddr, 32'h1);

    // pass letter to sim env
    i_sim_env.recv_letter(letter.addr, letter.data);

  endtask

  task automatic send_letter(input logic [31:0] addr, input logic [31:0] data);
    axi_write(IboxAddrAddr, addr);
    axi_write(IboxDataAddr, data);
    // set send and irq bits
    axi_write(AxiCtrlAddr, 32'h0000_0100);
  endtask

  task automatic raise_irq();
    axi_write(AxiCtrlAddr, 32'h0001_0000);
  endtask


endmodule : vip_mbx_driver
