module apb_cfg_regs #(
) (
    input logic clk_i,
    input logic rst_ni,
    APB.Slave apb_i
);

  logic [7:0] local_addr;
  assign local_addr = apb_i.paddr[7:0];

  // Bake in Git version information into HW
  logic [31:0] short_hash;
  assign short_hash = 32'h`GIT_HASH;

  // General-purpose register bank
  // Useful as white-box simulation hook
  logic [4:0][31:0] gpreg_d, gpreg_q;

  logic apb_event;
  assign apb_event = apb_i.psel & apb_i.penable;

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (~rst_ni) begin
      gpreg_q <= '0;
    end else begin
      gpreg_q <= gpreg_d;
    end
  end

  always_comb begin

    apb_i.prdata = 0;
    gpreg_d = gpreg_q;

    if (apb_event) begin
      if (apb_i.pwrite) begin
        // Currently supports only word writes
        unique case (local_addr)
          8'h00:  /*read-only*/;
          8'h04: gpreg_d[0] = apb_i.pwdata;
          8'h08: gpreg_d[1] = apb_i.pwdata;
          8'h0C: gpreg_d[2] = apb_i.pwdata;
          8'h10: gpreg_d[3] = apb_i.pwdata;
          8'h14: gpreg_d[4] = apb_i.pwdata;
          default: ;
        endcase
      end else begin
        unique case (local_addr)
          8'h00:   apb_i.prdata = short_hash;
          8'h04:   apb_i.prdata = gpreg_q[0];
          8'h08:   apb_i.prdata = gpreg_q[1];
          8'h0C:   apb_i.prdata = gpreg_q[2];
          8'h10:   apb_i.prdata = gpreg_q[3];
          8'h14:   apb_i.prdata = gpreg_q[4];
          default: ;
        endcase
      end
    end
  end

  //assign apb_i.prdata  = short_hash;
  assign apb_i.pready  = apb_i.psel & apb_i.penable;
  assign apb_i.pslverr = 1'b0;

endmodule : apb_cfg_regs

