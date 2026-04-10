module obi_mbx #(
) (
    input  logic               clk_i,
    input  logic               rst_ni,
    output logic               irq_o,
           OBI_BUS.Subordinate obi_sbr,
           AXI_LITE.Slave      axil_sbr
);

  localparam logic [31:0] StatAddr = 32'h0003_0000;
  localparam logic [31:0] ObiCtrlAddr = 32'h0003_0004;
  localparam logic [31:0] AxiCtrlAddr = 32'h0003_0008;
  localparam logic [31:0] IboxAddrAddr = 32'h0003_000C;
  localparam logic [31:0] IboxDataAddr = 32'h0003_0010;
  localparam logic [31:0] OboxAddrAddr = 32'h0003_0014;
  localparam logic [31:0] OboxDataAddr = 32'h0003_0018;

  localparam int unsigned InboxDepth = 3;
  localparam int unsigned OutboxDepth = 3;

  typedef struct packed {
    logic [31:0] addr;
    logic [31:0] data;
  } letter_t;

  letter_t out_letter_q, out_letter_d;
  letter_t in_letter_q, in_letter_d;

  logic out_letter_send;
  logic flush_ob, flush_ib;
  logic irq_clear, irq_set;

  logic inbox_full, inbox_empty;
  logic outbox_full, outbox_empty;
  logic obi_write_event, obi_read_event;

  logic aw_valid_q, ar_valid_q;

  logic [31:0] status_d, status_q;
  logic [31:0] obi_control_d, obi_control_q;
  logic [31:0] obi_rdata_d, obi_rdata_q;
  logic [31:0] axi_rdata_d, axi_rdata_q;

  assign obi_write_event = obi_sbr.req & obi_sbr.we;
  assign obi_read_event = obi_sbr.req & ~obi_sbr.we;

  assign status_d = {28'h0, outbox_full, outbox_empty, inbox_full, inbox_empty};

  assign axil_sbr.r_data = axi_rdata_q;

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      status_q         <= 32'h0;
      obi_control_q    <= 32'h0;
      ar_valid_q       <= 1'b0;
      aw_valid_q       <= 1'b0;
      obi_rdata_q      <= 32'h0;
      axi_rdata_q      <= 32'h0;
      out_letter_q     <= 'h0;
      in_letter_q      <= 'h0;
      axil_sbr.r_valid <= 1'b0;
    end else begin
      status_q         <= status_d;
      obi_control_q    <= obi_control_d;
      ar_valid_q       <= axil_sbr.ar_valid;
      aw_valid_q       <= axil_sbr.aw_valid;
      obi_rdata_q      <= obi_rdata_d;
      axi_rdata_q      <= axi_rdata_d;
      out_letter_q     <= out_letter_d;
      in_letter_q      <= in_letter_d;
      axil_sbr.r_valid <= axil_sbr.ar_valid;
    end
  end

  assign obi_sbr.gnt   = obi_sbr.req;
  assign obi_sbr.rdata = obi_rdata_q;

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      obi_sbr.rvalid <= 1'b0;
    end else begin
      obi_sbr.rvalid <= obi_sbr.gnt;
    end
  end

  always_ff @(posedge clk_i) begin
    if (~rst_ni) begin
      irq_o <= 1'b0;
    end else begin
      if (irq_clear) irq_o <= 1'b0;
      else if (irq_set) irq_o <= 1'b1;
      else irq_o <= irq_o;
    end
  end

  always_comb begin : addr_decode

    obi_rdata_d       = 32'h0;
    axi_rdata_d       = 32'h0;
    out_letter_send   = 1'b0;
    flush_ib          = 1'b0;
    flush_ob          = 1'b0;
    irq_clear         = 1'b0;
    irq_set           = 1'b0;

    axil_sbr.ar_ready = 0;

    obi_control_d     = obi_control_q;
    out_letter_d      = out_letter_q;
    in_letter_d       = in_letter_q;

    if (axil_sbr.aw_valid) begin
    end

    if (axil_sbr.ar_valid) begin
      axil_sbr.ar_ready = 1;
      axi_rdata_d = 32'h0b0110c5;
    end

    if (obi_write_event) begin
      unique case (obi_sbr.addr)
        ObiCtrlAddr: begin
          out_letter_send = obi_sbr.wdata[0];
          flush_ib        = obi_sbr.wdata[8];
          flush_ob        = obi_sbr.wdata[9];
          irq_set         = obi_sbr.wdata[16];
          irq_clear       = obi_sbr.wdata[17];
        end
        OboxAddrAddr: out_letter_d.addr = obi_sbr.wdata;
        OboxDataAddr: out_letter_d.data = obi_sbr.wdata;
        default: ;
      endcase
    end else if (obi_read_event) begin
      unique case (obi_sbr.addr)
        StatAddr:     obi_rdata_d = status_q;
        ObiCtrlAddr:  obi_rdata_d = obi_control_q;
        IboxAddrAddr: obi_rdata_d = in_letter_d.addr;
        IboxDataAddr: obi_rdata_d = in_letter_d.data;
        default:      ;
      endcase
    end
  end

  fifo_v3 #(
      .DEPTH(InboxDepth),
      .dtype(letter_t)
  ) i_inbox_fifo (
      .clk_i,
      .rst_ni,
      .flush_i   (flush_ib),
      .testmode_i(1'b0),
      .full_o    (inbox_full),
      .empty_o   (inbox_empty),
      .usage_o   (),
      .data_i    ('0),
      .push_i    (1'b0),
      .data_o    (),
      .pop_i     (1'b0)
  );
  fifo_v3 #(
      .DEPTH(OutboxDepth),
      .dtype(letter_t)
  ) i_outbox_fifo (
      .clk_i,
      .rst_ni,
      .flush_i   (flush_ob),
      .testmode_i(1'b0),
      .full_o    (outbox_full),
      .empty_o   (outbox_empty),
      .usage_o   (),
      .data_i    (out_letter_q),
      .push_i    (out_letter_send),
      .data_o    (),
      .pop_i     (1'b0)
  );

  // qverify tieoffs
  assign axil_sbr.aw_ready = 1'b0;
  assign axil_sbr.ar_ready = 1'b0;
  assign axil_sbr.w_ready = 1'b0;
  assign axil_sbr.b_resp = 'b0;
  assign axil_sbr.b_valid= 1'b0;
  assign axil_sbr.r_data = 32'h0;
  assign axil_sbr.r_resp = 'b0;
  assign axil_sbr.r_valid= 1'b0;

endmodule : obi_mbx

