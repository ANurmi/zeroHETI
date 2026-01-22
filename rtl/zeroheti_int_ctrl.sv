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
endmodule : zeroheti_int_ctrl
