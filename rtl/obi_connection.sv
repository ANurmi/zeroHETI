// Antti Nurmi <antti.nurmi@tuni.fi>
// Simplified OBI Mgr-Sbr connection with parameter
// for comb path cutting. Intentinally very verbose
// to preserve visibility and control of all signals.

module obi_connection #(
    parameter bit Cut = 1'b0
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Manager     obi_m,
          OBI_BUS.Subordinate obi_s
);

  if (Cut) begin : g_cut
    typedef struct packed {
      logic [31:0] addr;
      logic        we;
      logic [3:0]  be;
      logic [31:0] wdata;
    } a_chan_t;

    typedef struct packed {
      logic [31:0] rdata;
      logic        rvalid;
      logic        err;
    } r_chan_t;

    a_chan_t a_mgr, a_sbr;
    r_chan_t r_mgr, r_sbr;

    assign a_sbr.addr  = obi_s.addr;
    assign a_sbr.we    = obi_s.we;
    assign a_sbr.be    = obi_s.be;
    assign a_sbr.wdata = obi_s.wdata;

    assign obi_m.addr  = a_mgr.addr;
    assign obi_m.we    = a_mgr.we;
    assign obi_m.be    = a_mgr.be;
    assign obi_m.wdata = a_mgr.wdata;

    assign obi_s.rdata  = r_sbr.rdata;
    //assign obi_s.rvalid = r_sbr.rvalid;
    assign obi_s.err    = r_sbr.err;

    assign r_mgr.rdata  = obi_m.rdata;
    assign r_mgr.rvalid = obi_m.rvalid;
    assign r_mgr.err    = obi_m.err;

    spill_register #(
        .T(a_chan_t)
    ) i_reg_a (
        .clk_i,
        .rst_ni,
        .valid_i(obi_s.req),
        .ready_o(obi_s.gnt),
        .data_i (a_sbr),
        .valid_o(obi_m.req),
        .ready_i(obi_m.gnt),
        .data_o (a_mgr)
    );

    spill_register #(
        .T(r_chan_t)
    ) i_req_r (
        .clk_i,
        .rst_ni,
        .valid_i(obi_m.rvalid),
        .ready_o(),
        .data_i (r_mgr),
        .valid_o(obi_s.rvalid),
        .ready_i(1'b1),
        .data_o (r_sbr)
    );
  end : g_cut

  else begin : g_no_cut

    assign obi_m.req   = obi_s.req;
    assign obi_m.be    = obi_s.be;
    assign obi_m.we    = obi_s.we;
    assign obi_m.addr  = obi_s.addr;
    assign obi_m.wdata = obi_s.wdata;

    assign obi_s.gnt    = obi_m.gnt;
    assign obi_s.rdata  = obi_m.rdata;
    assign obi_s.rvalid = obi_m.rvalid;
    assign obi_s.err    = obi_m.err;

  end : g_no_cut

endmodule
