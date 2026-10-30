module zeroheti_int_ctrl #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::CpssCfg,
    localparam int unsigned NrIrqs = Cfg.irq.num_src,
    parameter int unsigned TsWidth = 12,
    localparam int unsigned IrqWidth = $clog2(Cfg.irq.num_src),
    localparam int unsigned PrioWidth = 8
) (
    input  logic                     clk_i,
    input  logic                     rst_ni,
    input  logic     [   NrIrqs-1:0] ext_irqs_i,
    output logic                     irq_valid_o,
    input  logic     [         63:0] mtime_i,
    input  logic                     mtime_en_i,
    output logic     [ IrqWidth-1:0] irq_id_o,
    input  logic     [ IrqWidth-1:0] irq_id_i,
    input  logic                     irq_ack_i,
    output logic                     irq_nest_o,
    output logic                     irq_heti_o,
    output logic     [PrioWidth-1:0] irq_level_o,
    output logic     [          1:0] irq_priv_o,
    output logic                     irq_shv_o,
           APB.Slave                 apb_sbr
);

  if (Cfg.irq.intc == zeroheti_pkg::EDFIC) begin : g_edfic

    logic [TsWidth-1:0] deadline;
    logic [7:0] dl_slice;
    logic [63:0] mtime;

    assign mtime = (mtime_en_i) ? mtime_i : 64'h0;

    // UPDATED: Extract bottom 8 bits
    // Adjust granularity with mtime prescaler
    assign dl_slice = deadline[7:0];

    assign irq_level_o = 8'd255 - dl_slice;

    // TODO: implement
    assign irq_heti_o = 1'b0;
    assign irq_nest_o = 1'b0;

    assign apb_sbr.pready = apb_sbr.psel & apb_sbr.penable;
    assign apb_sbr.pslverr = 1'b0;

    edfic_top #(
        .NrIrqs (NrIrqs),
        .TsWidth(TsWidth),
        .TsClip (32'd0)
    ) i_edfic (
        .clk_i,
        .rst_ni,
        .cfg_req_i  (apb_sbr.psel & apb_sbr.penable),
        .cfg_we_i   (apb_sbr.pwrite),
        .cfg_addr_i (apb_sbr.paddr),
        .cfg_wdata_i(apb_sbr.pwdata),
        .cfg_rdata_o(apb_sbr.prdata),
        .irq_i      (ext_irqs_i),
        .irq_id_o,
        .irq_id_i,
        .irq_dl_o   (deadline),
        .irq_valid_o,
        .irq_ack_i,
        .mtime_i    (mtime)
    );

    assign irq_priv_o = 2'b11;
    // EDFIC always uses selective hardware vectoring
    assign irq_shv_o  = 1'b1;

  end else if (Cfg.irq.intc == zeroheti_pkg::CLIC) begin : g_clic

    logic [7:0] irq_level;
    logic kill;
    assign irq_level_o = irq_level;  /*irq_level[IrqWidth-1:0];*/

    clic_apb #(
        .N_SOURCE(NrIrqs)
    ) i_clic (
        .clk_i,
        .rst_ni,
        .intr_src_i    (ext_irqs_i),
        .penable_i     (apb_sbr.penable),
        .pwrite_i      (apb_sbr.pwrite),
        .paddr_i       (apb_sbr.paddr),
        .psel_i        (apb_sbr.psel),
        .prdata_o      (apb_sbr.prdata),
        .pready_o      (apb_sbr.pready),
        .pslverr_o     (apb_sbr.pslverr),
        .pwdata_i      (apb_sbr.pwdata),
        .irq_priv_o,
        .irq_shv_o,
        .irq_id_o,
        .irq_valid_o,
        .irq_ready_i   (irq_ack_i),
        .irq_level_o   (irq_level),
        .irq_kill_ack_i(kill),
        .irq_kill_req_o(kill)
    );

  end

endmodule : zeroheti_int_ctrl
