module obi_mbx #(
) (
    input  logic               clk_i,
    input  logic               rst_ni,
    output logic               irq_o,
           OBI_BUS.Subordinate obi_sbr,
           AXI_LITE.Slave      axil_sbr
);

  localparam logic [31:0] StatAddr = 32'h0003_0000;
  localparam logic [31:0] CtrlAddr = 32'h0003_0004;
  localparam logic [31:0] IboxAddrAddr = 32'h0003_0008;
  localparam logic [31:0] IboxDataAddr = 32'h0003_000C;
  localparam logic [31:0] OboxAddrAddr = 32'h0003_0010;
  localparam logic [31:0] OboxDataAddr = 32'h0003_0014;

  localparam int unsigned InboxDepth = 3;
  localparam int unsigned OutboxDepth = 3;

  typedef struct packed {
    logic [31:0] addr;
    logic [31:0] data;
  } letter_t;

  letter_t out_letter_q, out_letter_d;
  letter_t in_letter_q, in_letter_d;

  logic out_letter_send;

  logic inbox_full, inbox_empty;
  logic outbox_full, outbox_empty;
  logic obi_write_event, obi_read_event;

  logic [31:0] status_d, status_q;
  logic [31:0] control_d, control_q;
  logic [31:0] rdata_d, rdata_q;

  assign obi_write_event = obi_sbr.req & obi_sbr.we;
  assign obi_read_event = obi_sbr.req & ~obi_sbr.we;

  assign status_d = {28'h0, outbox_full, outbox_empty, inbox_full, inbox_empty};

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      status_q     <= 32'h0;
      control_q    <= 32'h0;
      rdata_q      <= 32'h0;
      out_letter_q <= 'h0;
      in_letter_q  <= 'h0;
    end else begin
      status_q     <= status_d;
      control_q    <= control_d;
      rdata_q      <= rdata_d;
      out_letter_q <= out_letter_d;
      in_letter_q  <= in_letter_d;
    end
  end

  assign obi_sbr.gnt   = obi_sbr.req;
  assign obi_sbr.rdata = rdata_q;

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      obi_sbr.rvalid <= 1'b0;
    end else begin
      obi_sbr.rvalid <= obi_sbr.gnt;
    end
  end

  always_comb begin : obi_decode

    rdata_d      = 32'h0;
    out_letter_send = 1'b0;
    control_d    = control_q;
    out_letter_d = out_letter_q;
    in_letter_d  = in_letter_q;

    if (obi_write_event) begin
      unique case (obi_sbr.addr)
        CtrlAddr: begin
          out_letter_send = obi_sbr.wdata[0];
        end
        OboxAddrAddr: out_letter_d.addr = obi_sbr.wdata;
        OboxDataAddr: out_letter_d.data = obi_sbr.wdata;
        default: ;
      endcase
    end else if (obi_read_event) begin
      unique case (obi_sbr.addr)
        StatAddr:     rdata_d = status_q;
        CtrlAddr:     rdata_d = control_q;
        IboxAddrAddr: rdata_d = in_letter_d.addr;
        IboxDataAddr: rdata_d = in_letter_d.data;
        default:      ;
      endcase
    end
  end

  mbx_axi_ctrl i_axi_ctrl (
      .clk_i,
      .rst_ni,
      .axil_sbr
  );

  fifo_v3 #(
      .DEPTH(InboxDepth),
      .dtype(letter_t)
  ) i_inbox_fifo (
      .clk_i,
      .rst_ni,
      .flush_i   (),
      .testmode_i(),
      .full_o    (inbox_full),
      .empty_o   (inbox_empty),
      .usage_o   (),
      .data_i    (),
      .push_i    (),
      .data_o    (),
      .pop_i     ()
  );
  fifo_v3 #(
      .DEPTH(OutboxDepth),
      .dtype(letter_t)
  ) i_outbox_fifo (
      .clk_i,
      .rst_ni,
      .flush_i   (),
      .testmode_i(),
      .full_o    (outbox_full),
      .empty_o   (outbox_empty),
      .usage_o   (),
      .data_i    (out_letter_q),
      .push_i    (out_letter_send),
      .data_o    (),
      .pop_i     ()
  );

endmodule : obi_mbx

