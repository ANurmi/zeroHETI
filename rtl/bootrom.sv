//TODO: get rid of this eventually and replace with a functional, compiled ROM
// inspiration from https://github.com/pulp-platform/pulpissimo/tree/master/sw/bootcode

module bootrom #(
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Subordinate sbr_bus
);

  localparam int unsigned RomSize = 2;
  localparam int unsigned RomAddrWidth = $clog2(RomSize);

  localparam logic [31:0] BootRom[RomSize] = {32'h0000_006F, 32'h0000_006F};

  logic [RomAddrWidth-1:0] RomAddr;

  logic rvalid_q;

  // Assume zero-wait state memory -> rvalid follows req immediately
  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      rvalid_q <= 1'b0;
    end else begin
      rvalid_q <= sbr_bus.req;
    end
  end

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (~rst_ni) RomAddr <= '0;
    else RomAddr <= sbr_bus.addr[RomAddrWidth-1:0];
  end


  assign sbr_bus.rdata      = BootRom[RomAddr];


  assign sbr_bus.gnt        = sbr_bus.req;
  assign sbr_bus.rvalid     = rvalid_q;

  // Tie off unused parts
  assign sbr_bus.gntpar     = 1'b0;
  assign sbr_bus.rvalidpar  = 1'b0;
  assign sbr_bus.rid        = 1'b0;
  assign sbr_bus.r_optional = 1'b0;
  assign sbr_bus.err        = 1'b0;



endmodule : bootrom
