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
    $fatal(1, "edfic support not implemented yet");
  end else if (CoreCfg.ic == CLIC) begin : g_clic
    $fatal(1, "clic support not implemented yet");
  end

endmodule : zeroheti_int_ctrl
