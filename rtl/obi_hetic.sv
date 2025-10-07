module obi_hetic #(
  parameter  int unsigned NrIrqLines = 64,
  parameter  int unsigned NrIrqPrios = 32,
  localparam int unsigned IrqWidth   = $clog2(NrIrqLines),
  localparam int unsigned PrioWidth  = $clog2(NrIrqPrios)
)(
  input  logic                 clk_i,
  input  logic                 rst_ni,
  OBI_BUS.Subordinate          obi_sbr,
  output logic                 irq_valid_o,
  output logic                 irq_heti_o,
  output logic                 irq_nest_o,
  output logic [IrqWidth-1:0]  irq_id_o,
  input  logic [IrqWidth-1:0]  irq_id_i,
  input  logic                 irq_ack_i,
  output logic [PrioWidth-1:0] irq_level_o
);

localparam int unsigned PadWidth = 24-PrioWidth;

logic [10:0] line_idx;

logic write_event, read_event;

typedef struct packed {
  logic                 ie;
  logic                 ip;
  logic           [1:0] trig;
  logic                 heti;
  logic                 nest;
  logic [PrioWidth-1:0] prio;
} irq_line_t;

irq_line_t [NrIrqLines-1:0] lines_d, lines_q;

logic be_high;

logic [31:0] rdata_d, rdata_q;
logic        rvalid_q;

assign be_high        = obi_sbr.be[1:0] == 2'b00;
assign line_idx       = {obi_sbr.addr[11:2], be_high};
assign write_event    = obi_sbr.req &  obi_sbr.we;
assign read_event     = obi_sbr.req & ~obi_sbr.we;
assign obi_sbr.gnt    = obi_sbr.req;
assign obi_sbr.rvalid = rvalid_q;
assign obi_sbr.rdata  = rdata_q;

// Assume zero-wait state memory -> rvalid follows req immediately
always_ff @( posedge clk_i ) begin
  if (~rst_ni) begin
    rvalid_q <=  1'b0;
    rdata_q  <= 32'b0;
  end else begin
    rvalid_q <= obi_sbr.req;
    rdata_q  <= rdata_d;
  end
end

always_ff @(posedge clk_i) begin
  if (~rst_ni) begin
    lines_q <= '{default:0};
  end else begin
    lines_q <= lines_d;
  end
end

always_comb begin : main_comb

  lines_d  = lines_q;
  rdata_d  = rdata_q;

  if (read_event) begin : read
    if (be_high == 0) begin
      rdata_d = {PadWidth'('b0),
                  lines_q[line_idx].prio,
                  2'b0,
                  lines_q[line_idx].nest,
                  lines_q[line_idx].heti,
                  lines_q[line_idx].trig,
                  lines_q[line_idx].ip,
                  lines_q[line_idx].ie
                };
    end else begin
      rdata_d = {
                  8'(lines_q[line_idx].prio),
                  2'b0,
                  lines_q[line_idx].nest,
                  lines_q[line_idx].heti,
                  lines_q[line_idx].trig,
                  lines_q[line_idx].ip,
                  lines_q[line_idx].ie,
                  16'b0
                };
    end

  end : read
  else if (write_event) begin : write

    if (obi_sbr.be[3]) begin
      lines_d[line_idx].prio = obi_sbr.wdata[(PrioWidth+24-1):24];
    end
    if (obi_sbr.be[2]) begin
      lines_d[line_idx].nest = obi_sbr.wdata[21];
      lines_d[line_idx].heti = obi_sbr.wdata[20];
      lines_d[line_idx].trig = obi_sbr.wdata[19:18];
      lines_d[line_idx].ip   = obi_sbr.wdata[17];
      lines_d[line_idx].ie   = obi_sbr.wdata[16];
    end
    if (obi_sbr.be[1]) begin
      lines_d[line_idx].prio = obi_sbr.wdata[(PrioWidth+8-1):8];
    end
    if (obi_sbr.be[0]) begin
      lines_d[line_idx].nest = obi_sbr.wdata[5];
      lines_d[line_idx].heti = obi_sbr.wdata[4];
      lines_d[line_idx].trig = obi_sbr.wdata[3:2];
      lines_d[line_idx].ip   = obi_sbr.wdata[1];
      lines_d[line_idx].ie   = obi_sbr.wdata[0];
    end
  end : write

  // Claim acknowledged interrupt
  if (irq_ack_i) begin
    lines_d[irq_id_i].ip = 1'b0;
  end
end : main_comb

/*TODO: arb tree */

endmodule : obi_hetic

