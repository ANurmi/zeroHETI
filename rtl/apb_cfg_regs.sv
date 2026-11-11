module apb_cfg_regs #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::CLICCfg
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    output logic        intc_mtime_en_o,
    input  logic [31:0] paddr_i,
    input  logic [31:0] pwdata_i,
    output logic [31:0] prdata_o,
    output logic        pslverr_o,
    output logic        pready_o,
    input  logic        pwrite_i,
    input  logic        penable_i,
    input  logic        psel_i,
    input  logic [ 3:0] pstrb_i
);
  // Bake in Git version information into HW
  localparam logic [31:0] ShortHash = 32'h`GIT_HASH;
  // Static platform configuration visible to SW
  localparam logic IntcEdfic = (Cfg.irq.intc == zeroheti_pkg::EDFIC);
  localparam logic [7:0] ImemPow2 = 8'($clog2(Cfg.imem.bytes));
  localparam logic [7:0] DmemPow2 = 8'($clog2(Cfg.dmem.bytes));
`ifndef FULL_UART
  localparam logic RealUart = 1'b0;
`else
  localparam logic RealUart = 1'b1;
`endif
  localparam logic [31:0] PlatformCfg = {8'h0, DmemPow2, ImemPow2, 6'h0, RealUart, IntcEdfic};

  logic [11:0] local_addr;
  assign local_addr = paddr_i[11:0];


  // General-purpose register bank
  // Useful as white-box simulation hook
  logic [4:0][31:0] gpreg_d, gpreg_q;

  logic mtime_en_d, mtime_en_q;
  assign intc_mtime_en_o = mtime_en_q;

  logic apb_event;
  assign apb_event = psel_i & penable_i;

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (~rst_ni) begin
      gpreg_q    <= '0;
      mtime_en_q <= '0;
    end else begin
      gpreg_q    <= gpreg_d;
      mtime_en_q <= mtime_en_d;
    end
  end

  always_comb begin

    prdata_o   = 32'h0;
    gpreg_d    = gpreg_q;
    mtime_en_d = mtime_en_q;
    pslverr_o  = 1'b0;

    if (apb_event) begin
      if (pwrite_i) begin
        // Currently supports only word writes
        unique case (local_addr)
          12'h000:  /*read-only*/;
          12'h004:  /*read-only*/;
          12'h008: mtime_en_d = pwdata_i[0];
          12'h100: gpreg_d[0] = pwdata_i;
          12'h104: gpreg_d[1] = pwdata_i;
          12'h108: gpreg_d[2] = pwdata_i;
          12'h10C: gpreg_d[3] = pwdata_i;
          12'h110: gpreg_d[4] = pwdata_i;
          default: pslverr_o = 1'b1;
        endcase
      end else begin
        unique case (local_addr)
          12'h000: prdata_o = ShortHash;
          12'h004: prdata_o = PlatformCfg;
          12'h008: prdata_o = {31'h0, mtime_en_q};
          12'h100: prdata_o = gpreg_q[0];
          12'h104: prdata_o = gpreg_q[1];
          12'h108: prdata_o = gpreg_q[2];
          12'h10C: prdata_o = gpreg_q[3];
          12'h110: prdata_o = gpreg_q[4];
          default: pslverr_o = 1'b1;
        endcase
      end
    end
  end

  assign pready_o = psel_i & penable_i;


endmodule : apb_cfg_regs

