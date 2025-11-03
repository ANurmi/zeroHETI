module zeroheti_compliance #(
  parameter  zeroheti_pkg::core_cfg_t Cfg = zeroheti_pkg::`CORE_CFG
)(
  input  logic clk_i,
  input  logic rst_ni,
  output logic test_done_o
);

localparam int unsigned NumCpuPorts = 2;
localparam logic [31:0] ComplianceBootAddr = 32'h1_0000;

OBI_BUS cpu_bus [NumCpuPorts]();
OBI_BUS mem_bus ();

ibex_top_tracing #(
) i_rt_ibex (
  .clk_i,
  .rst_ni,

  .scan_rst_ni            (1'b0),
  .ram_cfg_i              (10'b0),
  .hart_id_i              (Cfg.hart_id),
  .test_en_i              (1'b0),
  .boot_addr_i            (ComplianceBootAddr),

  .instr_req_o            (cpu_bus[0].req),
  .instr_addr_o           (cpu_bus[0].addr),
  .instr_gnt_i            (cpu_bus[0].gnt),
  .instr_rvalid_i         (cpu_bus[0].rvalid),
  .instr_rdata_i          (cpu_bus[0].rdata),
  .instr_rdata_intg_i     (7'b0),
  .instr_err_i            (cpu_bus[0].err),

  .data_req_o             (cpu_bus[1].req),
  .data_gnt_i             (cpu_bus[1].gnt),
  .data_rvalid_i          (cpu_bus[1].rvalid),
  .data_we_o              (cpu_bus[1].we),
  .data_be_o              (cpu_bus[1].be),
  .data_addr_o            (cpu_bus[1].addr),
  .data_wdata_o           (cpu_bus[1].wdata),
  .data_rdata_i           (cpu_bus[1].rdata),
  .data_err_i             (cpu_bus[1].err),
  .data_rdata_intg_i      (7'b0),
  .data_wdata_intg_o      (),

  .irq_is_pcs_i (1'b0),
  .irq_i        ('0),
  .irq_id_o     (),
  .irq_level_i  ('0),
  .irq_shv_i    (1'b1),
  .irq_priv_i   (2'b11),
  .irq_ack_o    (),

  .scramble_key_valid_i   (1'b0),
  .scramble_key_i         (128'b0),
  .scramble_nonce_i       (64'b0),
  .scramble_req_o         (),

  .debug_req_i            (1'b0),
  .debug_mode_o           (),
  .crash_dump_o           (),
  .double_fault_seen_o    (),
  .fetch_enable_i         (4'b0101),
  .core_sleep_o           (),

  .alert_minor_o          (),
  .alert_major_internal_o (),
  .alert_major_bus_o      ()
);

obi_mux_intf #(
  .NumSbrPorts (NumCpuPorts),
  .NumMaxTrans (32'd1)
) i_mux (
  .clk_i,
  .rst_ni,
  .testmode_i (1'b0),
  .sbr_ports (cpu_bus),
  .mgr_port  (mem_bus)
);

obi_sram_intf #() i_mem (
  .clk_i,
  .rst_ni,
  .sbr (mem_bus)
);

endmodule : zeroheti_compliance

