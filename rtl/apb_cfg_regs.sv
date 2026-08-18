module apb_cfg_regs #(
) (
    input logic clk_i,
    input logic rst_ni,
    output logic intc_mtime_en_o,
    APB.Slave apb_i
);
  // Bake in Git version information into HW
  localparam logic [31:0] ShortHash = 32'h`GIT_HASH;
  // Static platform configuration visible to SW
  localparam logic IntcEdfic = (zeroheti_pkg::`INTC == zeroheti_pkg::EDFIC);
  localparam logic [7:0] ImemPow2 = 8'($clog2(`IMEM_BYTES));
  localparam logic [7:0] DmemPow2 = 8'($clog2(`DMEM_BYTES));
  localparam logic [31:0] PlatformCfg = {8'h0, DmemPow2, ImemPow2, 7'h0, IntcEdfic};

  logic [11:0] local_addr;
  assign local_addr = apb_i.paddr[11:0];


  // General-purpose register bank
  // Useful as white-box simulation hook
  logic [4:0][31:0] gpreg_d, gpreg_q;

  logic mtime_en_d, mtime_en_q;
  assign intc_mtime_en_o = mtime_en_q;

  logic apb_event;
  assign apb_event = apb_i.psel & apb_i.penable;

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

    apb_i.prdata = 0;
    gpreg_d      = gpreg_q;
    mtime_en_d   = mtime_en_q;

    if (apb_event) begin
      if (apb_i.pwrite) begin
        // Currently supports only word writes
        unique case (local_addr)
          12'h000:  /*read-only*/;
          12'h004:  /*read-only*/;
          12'h008: mtime_en_d = apb_i.pwdata[0];
          12'h100: gpreg_d[0] = apb_i.pwdata;
          12'h104: gpreg_d[1] = apb_i.pwdata;
          12'h108: gpreg_d[2] = apb_i.pwdata;
          12'h10C: gpreg_d[3] = apb_i.pwdata;
          12'h110: gpreg_d[4] = apb_i.pwdata;
          default: ;
        endcase
      end else begin
        unique case (local_addr)
          12'h000: apb_i.prdata = ShortHash;
          12'h004: apb_i.prdata = PlatformCfg;
          12'h008: apb_i.prdata = {31'h0, mtime_en_q};
          12'h100: apb_i.prdata = gpreg_q[0];
          12'h104: apb_i.prdata = gpreg_q[1];
          12'h108: apb_i.prdata = gpreg_q[2];
          12'h10C: apb_i.prdata = gpreg_q[3];
          12'h110: apb_i.prdata = gpreg_q[4];
          default: ;
        endcase
      end
    end
  end

  //assign apb_i.prdata  = short_hash;
  assign apb_i.pready  = apb_i.psel & apb_i.penable;
  assign apb_i.pslverr = 1'b0;

endmodule : apb_cfg_regs

