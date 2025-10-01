module zeroheti_dbg_wrapper import zeroheti_pkg::*; #()(
  input  logic clk_i,
  input  logic rst_ni,
  input  logic testmode_i,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o,
  output logic ndmreset_o,
  output logic debug_req_o,
  OBI_BUS.Subordinate mem_sbr,
  OBI_BUS.Manager     sba_mgr
);

logic          dmi_rst_n;
dm::dmi_req_t  dmi_req;
logic          dmi_req_ready, dmi_req_valid;
dm::dmi_resp_t dmi_rsp;
logic          dmi_rsp_ready, dmi_rsp_valid;

dmi_jtag #(
  .IdcodeValue ( 32'hFEEDC0D3 )
) i_dmi_jtag (
  .clk_i,
  .rst_ni,
  .testmode_i,
  .dmi_rst_no       (dmi_rst_n),
  .dmi_req_valid_o  (dmi_req_valid),
  .dmi_req_ready_i  (dmi_req_ready),
  .dmi_req_o        (dmi_req),
  .dmi_resp_valid_i (dmi_rsp_valid),
  .dmi_resp_ready_o (dmi_rsp_ready),
  .dmi_resp_i       (dmi_rsp),
  .tck_i            (jtag_tck_i),
  .tms_i            (jtag_tms_i),
  .trst_ni          (jtag_trst_ni),
  .td_i             (jtag_td_i),
  .td_o             (jtag_td_o),
  .tdo_oe_o         ()
);

dm_obi_top #(
  .DmBaseAddress (AddrMap.dbg.base)
) i_dm_obi_top (
  .clk_i,
  .rst_ni,
  .testmode_i,
  .ndmreset_o         (),
  .dmactive_o         (),
  .debug_req_o        (),
  .unavailable_i      (),
  .hartinfo_i         (),

  .dmi_rst_ni         (dmi_rst_n),
  .dmi_req_valid_i    (dmi_req_valid),
  .dmi_req_ready_o    (dmi_req_ready),
  .dmi_req_i          (dmi_req),
  .dmi_resp_valid_o   (dmi_rsp_valid),
  .dmi_resp_ready_i   (dmi_rsp_ready),
  .dmi_resp_o         (dmi_rsp),

  .slave_req_i        (mem_sbr.req),
  .slave_gnt_o        (mem_sbr.gnt),
  .slave_addr_i       (mem_sbr.addr),
  .slave_we_i         (mem_sbr.we),
  .slave_be_i         (mem_sbr.be),
  .slave_wdata_i      (mem_sbr.wdata),
  .slave_aid_i        (mem_sbr.aid),
  .slave_rvalid_o     (mem_sbr.rvalid),
  .slave_rdata_o      (mem_sbr.rdata),
  .slave_rid_o        (mem_sbr.rid),

  .master_req_o       (),
  .master_addr_o      (),
  .master_we_o        (),
  .master_wdata_o     (),
  .master_be_o        (),
  .master_gnt_i       (),
  .master_rdata_i     (),
  .master_rvalid_i    (),
  .master_err_i       (),
  .master_other_err_i ()
);

endmodule : zeroheti_dbg_wrapper

