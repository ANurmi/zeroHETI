module zeroheti_compliance #(
    parameter  zeroheti_pkg::core_cfg_t Cfg = zeroheti_pkg::`CORE_CFG
) (
    input  logic        clk_i,
    input  logic        rst_ni,
    // Instruction memory interface
    output logic        instr_req_o,
    input  logic        instr_gnt_i,
    input  logic        instr_rvalid_i,
    output logic [31:0] instr_addr_o,
    input  logic [31:0] instr_rdata_i,
    input  logic        instr_err_i,
    // Data memory interface
    output logic        data_req_o,
    input  logic        data_gnt_i,
    input  logic        data_rvalid_i,
    output logic        data_we_o,
    output logic [ 3:0] data_be_o,
    output logic [31:0] data_addr_o,
    output logic [31:0] data_wdata_o,
    input  logic [31:0] data_rdata_i,
    input  logic        data_err_i,
    // Not used, but needed for vbench conformance
    input  logic jtag_tck_i,
    input  logic jtag_tms_i,
    input  logic jtag_td_i,
    input  logic jtag_trst_ni,
    output logic jtag_td_o
);

  localparam logic [31:0] ComplianceBootAddr = 32'h0000_0000;

  ibex_top_tracing #() i_rt_ibex (
      .clk_i,
      .rst_ni,

      .scan_rst_ni(1'b0),
      .ram_cfg_i  (10'b0),
      .hart_id_i  (Cfg.hart_id),
      .test_en_i  (1'b0),
      .boot_addr_i(ComplianceBootAddr),

      .instr_req_o,
      .instr_addr_o,
      .instr_gnt_i,
      .instr_rvalid_i,
      .instr_rdata_i,
      .instr_rdata_intg_i(7'b0),
      .instr_err_i,

      .data_req_o,
      .data_gnt_i,
      .data_rvalid_i,
      .data_we_o,
      .data_be_o,
      .data_addr_o,
      .data_wdata_o,
      .data_rdata_i,
      .data_err_i,
      .data_rdata_intg_i(7'b0),
      .data_wdata_intg_o(),

      .irq_is_pcs_i(1'b0),
      .irq_i       ('0),
      .irq_id_o    (),
      .irq_level_i ('0),
      .irq_shv_i   (1'b1),
      .irq_priv_i  (2'b11),
      .irq_ack_o   (),

      .scramble_key_valid_i(1'b0),
      .scramble_key_i      (128'b0),
      .scramble_nonce_i    (64'b0),
      .scramble_req_o      (),

      .debug_req_i        (1'b0),
      .debug_mode_o       (),
      .crash_dump_o       (),
      .double_fault_seen_o(),
      .fetch_enable_i     (4'b0101),
      .core_sleep_o       (),

      .alert_minor_o         (),
      .alert_major_internal_o(),
      .alert_major_bus_o     ()
  );

endmodule : zeroheti_compliance

