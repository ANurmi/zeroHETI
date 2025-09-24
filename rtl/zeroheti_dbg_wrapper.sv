module zeroheti_dbg_wrapper #()(
  input  logic clk_i,
  input  logic rst_ni
);

dmi_jtag #(
  .IdcodeValue ( 32'hFEEDC0D3 )
) i_dmi_jtag (
  .clk_i            (),
  .rst_ni           (),
  .testmode_i       (),
  .dmi_rst_no       (),
  .dmi_req_valid_o  (),
  .dmi_req_ready_i  (),
  .dmi_req_o        (),
  .dmi_resp_valid_i (),
  .dmi_resp_ready_o (),
  .dmi_resp_i       (),
  .tck_i            (),
  .tms_i            (),
  .trst_ni          (),
  .td_i             (),
  .td_o             (),
  .tdo_oe_o         ()
);

dm_obi_top #() i_dm_obi_top (
  .clk_i,
  .rst_ni,
  .testmode_i         (),
  .ndmreset_o         (),
  .dmactive_o         (),
  .debug_req_o        (),
  .unavailable_i      (),
  .hartinfo_i         (),

  .dmi_rst_ni         (),
  .dmi_req_valid_i    (),
  .dmi_req_ready_o    (),
  .dmi_req_i          (),
  .dmi_resp_valid_o   (),
  .dmi_resp_ready_i   (),
  .dmi_resp_o         (),

  .slave_req_i        (),
  .slave_gnt_o        (),
  .slave_addr_i       (),
  .slave_we_i         (),
  .slave_be_i         (),
  .slave_wdata_i      (),
  .slave_aid_i        (),
  .slave_rvalid_o     (),
  .slave_rdata_o      (),
  .slave_rid_o        (),

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

