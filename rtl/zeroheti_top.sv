module zeroheti_top
  import zeroheti_pkg::AddrMap;
  import zeroheti_pkg::TGSize;
#(
    parameter zeroheti_pkg::core_cfg_t CoreCfg = zeroheti_pkg::`CORE_CFG,
    parameter int unsigned AxiUserWidth = 0,
    localparam int unsigned NumIntIrqs = 27,
    localparam int unsigned NumExtIrqs = CoreCfg.num_irqs - NumIntIrqs
) (
    input  logic                  clk_i,
    input  logic                  rst_ni,
    input  logic                  jtag_tck_i,
    input  logic                  jtag_tms_i,
    input  logic                  jtag_trst_ni,
    input  logic                  jtag_td_i,
    output logic                  jtag_td_o,
    input  logic [NumExtIrqs-1:0] ext_irq_i,
    input  logic [          31:0] sbr_axil_aw_addr_i,
    input  logic [           2:0] sbr_axil_aw_prot_i,
    input  logic                  sbr_axil_aw_valid_i,
    output logic                  sbr_axil_aw_ready_o,
    input  logic [          31:0] sbr_axil_w_data_i,
    input  logic [           3:0] sbr_axil_w_strb_i,
    input  logic                  sbr_axil_w_valid_i,
    output logic                  sbr_axil_w_ready_o,
    output logic [           1:0] sbr_axil_b_resp_o,
    output logic                  sbr_axil_b_valid_o,
    input  logic                  sbr_axil_b_ready_i,
    input  logic [          31:0] sbr_axil_ar_addr_i,
    input  logic [           2:0] sbr_axil_ar_prot_i,
    input  logic                  sbr_axil_ar_valid_i,
    output logic                  sbr_axil_ar_ready_o,
    output logic [          31:0] sbr_axil_r_data_o,
    output logic [           1:0] sbr_axil_r_resp_o,
    output logic                  sbr_axil_r_valid_o,
    input  logic                  sbr_axil_r_ready_i,
    output logic [          31:0] mgr_axil_aw_addr_o,
    output logic [           2:0] mgr_axil_aw_prot_o,
    output logic                  mgr_axil_aw_valid_o,
    input  logic                  mgr_axil_aw_ready_i,
    output logic [          31:0] mgr_axil_w_data_o,
    output logic [           3:0] mgr_axil_w_strb_o,
    output logic                  mgr_axil_w_valid_o,
    input  logic                  mgr_axil_w_ready_i,
    input  logic [           1:0] mgr_axil_b_resp_i,
    input  logic                  mgr_axil_b_valid_i,
    output logic                  mgr_axil_b_ready_o,
    output logic [          31:0] mgr_axil_ar_addr_o,
    output logic [           2:0] mgr_axil_ar_prot_o,
    output logic                  mgr_axil_ar_valid_o,
    input  logic                  mgr_axil_ar_ready_i,
    input  logic [          31:0] mgr_axil_r_data_i,
    input  logic [           1:0] mgr_axil_r_resp_i,
    input  logic                  mgr_axil_r_valid_i,
    output logic                  mgr_axil_r_ready_o,
    input  logic                  uart_rx_i,
    output logic                  uart_tx_o,
    input  logic                  i2c_scl_pad_i,
    output logic                  i2c_scl_pad_o,
    output logic                  i2c_scl_padoen_o,
    input  logic                  i2c_sda_pad_i,
    output logic                  i2c_sda_pad_o,
    output logic                  i2c_sda_padoen_o
);

  localparam int unsigned NrIrqs = CoreCfg.num_irqs;
  localparam int unsigned ApbWidth = 32;
  localparam int unsigned DataWidth = 32;
  localparam int unsigned NrApbPerip = 5;
  localparam int unsigned SelWidth = $clog2(NrApbPerip);

  OBI_BUS obi_mgr ();
  OBI_BUS mbx_mgr ();
  OBI_BUS obi_sbr ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_sbr ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_mgr ();

  APB apb_mgr ();
  APB demux_apb[NrApbPerip] ();

  logic                  obi_sel;
  logic [  SelWidth-1:0] demux_sel;
  logic [    NrIrqs-1:0] all_irqs;
  logic                  mtime_irq;
  logic                  i2c_irq;
  logic                  mbx_irq;
  logic                  uart_irq;
  logic [(TGSize*2)-1:0] apb_timer_irqs;

  logic [          63:0] mtime;
  logic                  intc_mtime_en;

  always_comb begin : irq_mapping
    all_irqs                       = '0;
    all_irqs[3]                    = '0;  // legacy sw irq
    all_irqs[11]                   = ext_irq_i[0];  // legacy ext irq
    all_irqs[7]                    = mtime_irq;  // legacy tmr irq
    all_irqs[((2*TGSize)+16)-1:16] = apb_timer_irqs;
    all_irqs[24]                   = uart_irq;
    all_irqs[25]                   = i2c_irq;
    all_irqs[26]                   = mbx_irq;
    all_irqs[NrIrqs-1:27]          = ext_irq_i;
    //all_irqs[31]                 = nmi, reserved;
  end : irq_mapping

  zeroheti_core #(
      .Cfg(CoreCfg)
  ) i_core (
      .clk_i,
      .rst_ni,
      .testmode_i     (1'b0),
      .mtime_i        (mtime),
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .intc_mtime_en_i(intc_mtime_en),
      .ext_irqs_i     (all_irqs),
      .obi_mgr        (obi_mgr),
      .mbx_mgr        (mbx_mgr),
      .apb_mgr        (apb_mgr),
      .obi_sbr        (obi_sbr)
  );

  obi_to_axi_lite_intf #(
      .AxiAddrWidth(32'd32),
      .AxiDataWidth(32'd32),
      .AxiUserWidth(AxiUserWidth)
  ) i_obi_to_axi (
      .clk_i,
      .rst_ni,
      .obi_sbr(obi_mgr),
      .axi_mgr(axi_mgr)
  );

  obi_mbx i_mbx (
      .clk_i,
      .rst_ni,
      .irq_o   (mbx_irq),
      .obi_sbr (mbx_mgr),
      .axil_sbr(axi_sbr)
  );

  always_comb begin : apb_decode
    unique case (apb_mgr.paddr) inside
      [AddrMap.cfg.base : AddrMap.cfg.last - 1]:       demux_sel = SelWidth'('d0);
      [AddrMap.tg.base : AddrMap.tg.last - 1]:         demux_sel = SelWidth'('d1);
      [AddrMap.uart.base : AddrMap.uart.last - 1]:     demux_sel = SelWidth'('d2);
      [AddrMap.mtimer.base : AddrMap.mtimer.last - 1]: demux_sel = SelWidth'('d3);
      [AddrMap.i2c.base : AddrMap.i2c.last - 1]:       demux_sel = SelWidth'('d4);
      default: begin
        demux_sel = SelWidth'('d0);
        if (apb_mgr.psel & apb_mgr.penable) $display("Warning: APB access to unmapped region!");
      end
    endcase
  end

  apb_demux_intf #(
      .APB_ADDR_WIDTH(ApbWidth),
      .APB_DATA_WIDTH(DataWidth),
      .NoMstPorts    (NrApbPerip)
  ) i_apb_demux (
      .slv     (apb_mgr),
      .mst     (demux_apb),
      .select_i(demux_sel)
  );

  uart_wrapper #() i_uart (
      .clk_i,
      .rst_ni,
      .apb_sbr(demux_apb[2]),
      .rx_i   (uart_rx_i),
      .tx_o   (uart_tx_o),
      .irq_o  (uart_irq)
  );

  apb_mtimer i_mtimer (
      .clk_i,
      .rst_ni,
      .penable_i  (demux_apb[1].penable),
      .pwrite_i   (demux_apb[1].pwrite),
      .paddr_i    (demux_apb[1].paddr),
      .psel_i     (demux_apb[1].psel),
      .pwdata_i   (demux_apb[1].pwdata),
      .prdata_o   (demux_apb[1].prdata),
      .pready_o   (demux_apb[1].pready),
      .pslverr_o  (demux_apb[1].pslverr),
      .mtime_o    (mtime),
      .timer_irq_o(mtime_irq)
  );

  apb_timer #(
      .APB_ADDR_WIDTH(ApbWidth),
      .TIMER_CNT(TGSize)
  ) i_apb_timer (
      .HCLK   (clk_i),
      .HRESETn(rst_ni),
      .PENABLE(demux_apb[3].penable),
      .PWRITE (demux_apb[3].pwrite),
      .PADDR  (demux_apb[3].paddr),
      .PSEL   (demux_apb[3].psel),
      .PWDATA (demux_apb[3].pwdata),
      .PRDATA (demux_apb[3].prdata),
      .PREADY (demux_apb[3].pready),
      .PSLVERR(demux_apb[3].pslverr),
      .irq_o  (apb_timer_irqs)
  );

  apb_i2c #(
      .APB_ADDR_WIDTH(32'd32)
  ) i_i2c (
      .HCLK        (clk_i),
      .HRESETn     (rst_ni),
      .PADDR       (demux_apb[0].paddr),
      .PWDATA      (demux_apb[0].pwdata),
      .PWRITE      (demux_apb[0].pwrite),
      .PSEL        (demux_apb[0].psel),
      .PENABLE     (demux_apb[0].penable),
      .PRDATA      (demux_apb[0].prdata),
      .PREADY      (demux_apb[0].pready),
      .PSLVERR     (demux_apb[0].pslverr),
      .interrupt_o (i2c_irq),
      .scl_pad_i   (i2c_scl_pad_i),
      .scl_pad_o   (i2c_scl_pad_o),
      .scl_padoen_o(i2c_scl_padoen_o),
      .sda_pad_i   (i2c_sda_pad_i),
      .sda_pad_o   (i2c_sda_pad_o),
      .sda_padoen_o(i2c_sda_padoen_o)
  );

  apb_cfg_regs #() i_cfg_regs (
      .clk_i,
      .rst_ni,
      .intc_mtime_en_o(intc_mtime_en),
      .apb_i(demux_apb[4])
  );

  assign axi_sbr.aw_valid    = sbr_axil_aw_valid_i;
  assign axi_sbr.aw_addr     = sbr_axil_aw_addr_i;
  assign axi_sbr.aw_prot     = sbr_axil_aw_prot_i;
  assign sbr_axil_aw_ready_o = axi_sbr.aw_ready;

  assign axi_sbr.w_valid     = sbr_axil_w_valid_i;
  assign axi_sbr.w_data      = sbr_axil_w_data_i;
  assign axi_sbr.w_strb      = sbr_axil_w_strb_i;
  assign sbr_axil_w_ready_o  = axi_sbr.w_ready;

  assign sbr_axil_b_resp_o   = axi_sbr.b_resp;
  assign sbr_axil_b_valid_o  = axi_sbr.b_valid;
  assign axi_sbr.b_ready     = sbr_axil_b_ready_i;

  assign axi_sbr.ar_valid    = sbr_axil_ar_valid_i;
  assign axi_sbr.ar_addr     = sbr_axil_ar_addr_i;
  assign axi_sbr.ar_prot     = sbr_axil_ar_prot_i;
  assign sbr_axil_ar_ready_o = axi_sbr.ar_ready;

  assign sbr_axil_r_data_o   = axi_sbr.r_data;
  assign sbr_axil_r_resp_o   = axi_sbr.r_resp;
  assign sbr_axil_r_valid_o  = axi_sbr.r_valid;
  assign axi_sbr.r_ready     = sbr_axil_r_ready_i;


  assign mgr_axil_aw_addr_o  = axi_mgr.aw_addr;
  assign mgr_axil_aw_valid_o = axi_mgr.aw_valid;
  assign mgr_axil_aw_prot_o  = axi_mgr.aw_prot;
  assign axi_mgr.aw_ready    = mgr_axil_aw_ready_i;

  assign mgr_axil_ar_addr_o  = axi_mgr.ar_addr;
  assign mgr_axil_ar_valid_o = axi_mgr.ar_valid;
  assign mgr_axil_ar_prot_o  = axi_mgr.ar_prot;
  assign axi_mgr.ar_ready    = mgr_axil_ar_ready_i;

  assign mgr_axil_w_data_o   = axi_mgr.w_data;
  assign mgr_axil_w_strb_o   = axi_mgr.w_strb;
  assign mgr_axil_w_valid_o  = axi_mgr.w_valid;
  assign axi_mgr.w_ready     = mgr_axil_w_ready_i;

  assign axi_mgr.b_valid     = mgr_axil_b_valid_i;
  assign axi_mgr.b_resp      = mgr_axil_b_resp_i;
  assign mgr_axil_b_ready_o  = axi_mgr.b_ready;

  assign axi_mgr.r_data      = mgr_axil_r_data_i;
  assign axi_mgr.r_resp      = mgr_axil_r_resp_i;
  assign axi_mgr.r_valid     = mgr_axil_r_valid_i;
  assign mgr_axil_r_ready_o  = axi_mgr.r_ready;


  assign obi_sbr.addr        = '0;
  assign obi_sbr.req         = 1'b0;
  assign obi_sbr.rready      = 1'b0;
  assign obi_sbr.wdata       = '0;
  assign obi_sbr.be          = '0;
  assign obi_sbr.we          = 1'b0;
  assign obi_sbr.aid         = 1'b0;
  assign obi_sbr.a_optional  = 1'b0;
  assign obi_sbr.reqpar      = 1'b0;

`ifndef SYNTHESIS
`ifndef TECH_MEMORY

  `define STR(s) `"s`"

  typedef enum bit {
    JTAG,
    READMEM
  } load_e;
  load_e LoadType;
  string zeroHetiRoot = `STR(`ZH_ROOT);

  initial begin : simulation_loader

    LoadType = `LOAD;

    if (LoadType == READMEM) begin
      @(posedge rst_ni);
      $display("[DUT:SimLoader] Initializing program with $readmemh");
      $display("[DUT:SimLoader] APPLICABLE TO SIMULATED DESIGNS ONLY");

      // Preload 4 IMEM banks
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_0.hex"},
                  i_core.i_imem.g_banks[0].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_1.hex"},
                  i_core.i_imem.g_banks[1].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_2.hex"},
                  i_core.i_imem.g_banks[2].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_3.hex"},
                  i_core.i_imem.g_banks[3].i_sram.sram);

      // Preload 2 DMEM banks
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_0.hex"},
                  i_core.i_dmem.g_banks[0].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_1.hex"},
                  i_core.i_dmem.g_banks[1].i_sram.sram);

    end
  end

`endif
`endif


endmodule : zeroheti_top

