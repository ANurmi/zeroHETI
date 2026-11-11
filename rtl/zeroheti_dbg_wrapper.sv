module zeroheti_dbg_wrapper #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::CLICCfg
) (
    input  logic           clk_i,
    input  logic           rst_ni,
    input  logic           testmode_i,
    input  logic           jtag_tck_i,
    input  logic           jtag_tms_i,
    input  logic           jtag_trst_ni,
    input  logic           jtag_td_i,
    output logic           jtag_td_o,
    output logic           ndmreset_o,
    output logic           debug_req_o,
           AXI_LITE.Slave  mem_sbr,
           AXI_LITE.Master sba_mgr
);

  localparam int unsigned IdCode = Cfg.jtag_idcode;

  localparam int unsigned AddrWidth = 32;
  localparam int unsigned DataWidth = 32;

  typedef logic [AddrWidth-1:0] addr_t;
  typedef logic [DataWidth-1:0] data_t;
  typedef logic [DataWidth/8-1:0] strb_t;

  logic         dmi_rst_n;
  dm::dmi_req_t dmi_req;
  logic dmi_req_ready, dmi_req_valid;
  dm::dmi_resp_t dmi_rsp;
  logic dmi_rsp_ready, dmi_rsp_valid;

  // Debug Module slave memory port
  logic dbg_req, dbg_we;
  addr_t dbg_addr;
  data_t dbg_wdata, dbg_rdata;
  strb_t dbg_strb;

  // Debug Module System Bus Access manager port
  logic sba_req, sba_we, sba_gnt, sba_rvalid, sba_err;
  addr_t sba_addr;
  data_t sba_wdata, sba_rdata;
  strb_t sba_strb;

  dmi_jtag #(
      .IdcodeValue(IdCode)
  ) i_dmi_jtag (
      .clk_i,
      .rst_ni,
      .testmode_i,
      .dmi_rst_no      (dmi_rst_n),
      .dmi_req_valid_o (dmi_req_valid),
      .dmi_req_ready_i (dmi_req_ready),
      .dmi_req_o       (dmi_req),
      .dmi_resp_valid_i(dmi_rsp_valid),
      .dmi_resp_ready_o(dmi_rsp_ready),
      .dmi_resp_i      (dmi_rsp),
      .tck_i           (jtag_tck_i),
      .tms_i           (jtag_tms_i),
      .trst_ni         (jtag_trst_ni),
      .td_i            (jtag_td_i),
      .td_o            (jtag_td_o),
      .tdo_oe_o        ()
  );

  dm_top #(
      .DmBaseAddress(Cfg.am.core.dbg.base)
  ) i_dm_obi_top (
      .clk_i,
      .rst_ni,
      .testmode_i,
      .ndmreset_o,
      .dmactive_o   (),
      .debug_req_o,
      .unavailable_i('0),
      .hartinfo_i   ('0),

      .dmi_rst_ni      (dmi_rst_n),
      .dmi_req_valid_i (dmi_req_valid),
      .dmi_req_ready_o (dmi_req_ready),
      .dmi_req_i       (dmi_req),
      .dmi_resp_valid_o(dmi_rsp_valid),
      .dmi_resp_ready_i(dmi_rsp_ready),
      .dmi_resp_o      (dmi_rsp),

      .slave_req_i  (dbg_req),
      .slave_we_i   (dbg_we),
      .slave_addr_i (dbg_addr),
      .slave_be_i   (dbg_strb),
      .slave_wdata_i(dbg_wdata),
      .slave_rdata_o(dbg_rdata),

      .master_req_o        (sba_req),
      .master_add_o        (sba_addr),
      .master_we_o         (sba_we),
      .master_wdata_o      (sba_wdata),
      .master_be_o         (sba_strb),
      .master_gnt_i        (sba_gnt),
      .master_r_valid_i    (sba_rvalid),
      .master_r_err_i      (sba_err),
      .master_r_other_err_i(1'b0),
      .master_r_rdata_i    (sba_rdata)
  );

  //////////////////////////////////////////
  // Debug Module slave (AXI-Lite -> mem) //
  //////////////////////////////////////////

  axi_lite_to_mem #(
      .AddrWidth(AddrWidth),
      .DataWidth(DataWidth)
  ) i_dbg_axi_lite_to_mem (
      .clk_i,
      .rst_ni,
      .slv        (mem_sbr),
      .mem_req_o  (dbg_req),
      .mem_we_o   (dbg_we),
      .mem_addr_o (dbg_addr),
      .mem_wdata_o(dbg_wdata),
      .mem_be_o   (dbg_strb),
      .mem_rdata_i(dbg_rdata)
  );

  ///////////////////////////////////////////////
  // Debug System Bus Access (mem -> AXI-Lite) //
  ///////////////////////////////////////////////

  axi_lite_from_mem_intf #(
      .MEM_ADDR_WIDTH(AddrWidth),
      .AXI_ADDR_WIDTH(AddrWidth),
      .DATA_WIDTH    (DataWidth),
      .MAX_REQUESTS  (1),          // The DM SBA keeps at most one request in flight
      .AXI_PROT      ('0)
  ) i_dbg_sba_axi_lite_from_mem (
      .clk_i,
      .rst_ni,
      .mem_req_i      (sba_req),
      .mem_addr_i     (sba_addr),
      .mem_we_i       (sba_we),
      .mem_wdata_i    (sba_wdata),
      .mem_be_i       (sba_strb),
      .mem_gnt_o      (sba_gnt),
      .mem_rsp_valid_o(sba_rvalid),
      .mem_rsp_rdata_o(sba_rdata),
      .mem_rsp_error_o(sba_err),
      .mst            (sba_mgr)
  );

endmodule : zeroheti_dbg_wrapper
