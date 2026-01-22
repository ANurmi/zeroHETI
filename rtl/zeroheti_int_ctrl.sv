module zeroheti_int_ctrl #(
    parameter  zeroheti_pkg::core_cfg_t CoreCfg   = zeroheti_pkg::DefaultCfg,
    localparam int unsigned             NrIrqs    = CoreCfg.num_irqs,
    localparam int unsigned             IrqWidth  = $clog2(CoreCfg.num_irqs),
    localparam int unsigned             PrioWidth = $clog2(CoreCfg.num_prio)
) (
    input  logic                               clk_i,
    input  logic                               rst_ni,
    input  logic               [   NrIrqs-1:0] ext_irqs_i,
    output logic                               irq_valid_o,
    output logic               [ IrqWidth-1:0] irq_id_o,
    input  logic               [ IrqWidth-1:0] irq_id_i,
    input  logic                               irq_ack_i,
    output logic                               irq_nest_o,
    output logic                               irq_heti_o,
    output logic               [PrioWidth-1:0] irq_level_o,
    output logic               [          1:0] irq_priv_o,
    output logic                               irq_shv_o,
           OBI_BUS.Subordinate                 obi_sbr
);
  import zeroheti_pkg::*;

  if (CoreCfg.ic == HETIC) begin : g_hetic

    obi_hetic #(
        .NrIrqLines(CoreCfg.num_irqs),
        .NrIrqPrios(CoreCfg.num_prio)
    ) i_hetic (
        .clk_i,
        .rst_ni,
        .ext_irqs_i,
        .irq_heti_o,
        .irq_nest_o,
        .irq_id_o,
        .irq_valid_o,
        .irq_id_i,
        .irq_ack_i,
        .irq_level_o,
        .obi_sbr
    );

    assign irq_priv_o = 2'b11;
    assign irq_shv_o  = 1'b1;

  end else if (CoreCfg.ic == EDFIC) begin : g_edfic

    APB ic_apb ();

    obi_to_apb_intf i_obi_to_apb (
        .clk_i,
        .rst_ni,
        .obi_i(obi_sbr),
        .apb_o(ic_apb)
    );

    edf_ic #(
    ) i_edfic ();

  end else if (CoreCfg.ic == CLIC) begin : g_clic

    logic [7:0] irq_level;
    assign irq_level_o = irq_level[IrqWidth-1:0];

    APB ic_apb ();

    obi_to_apb_intf i_obi_to_apb (
        .clk_i,
        .rst_ni,
        .obi_i(obi_sbr),
        .apb_o(ic_apb)
    );

    clic_apb #(
        .N_SOURCE(NrIrqs)
    ) i_clic (
        .clk_i,
        .rst_ni,
        .intr_src_i    (ext_irqs_i),
        .penable_i     (ic_apb.penable),
        .pwrite_i      (ic_apb.pwrite),
        .paddr_i       (ic_apb.paddr),
        .psel_i        (ic_apb.psel),
        .prdata_o      (ic_apb.prdata),
        .pready_o      (ic_apb.pready),
        .pslverr_o     (ic_apb.pslverr),
        .pwdata_i      (ic_apb.pwdata),
        .irq_priv_o,
        .irq_shv_o,
        .irq_id_o,
        .irq_valid_o,
        .irq_ready_i   (irq_ack_i),
        .irq_level_o   (irq_level),
        .irq_kill_ack_i(1'b0),
        .irq_kill_req_o()
    );

  end

endmodule : zeroheti_int_ctrl
