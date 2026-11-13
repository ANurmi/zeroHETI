module zeroheti_top #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::EDFICCfg,
    parameter int unsigned AxiUserWidth = 0,
    localparam int unsigned NumIntIrqs = 64,
    localparam int unsigned NumExtIrqs = Cfg.irq.num_src - NumIntIrqs
) (
    input  logic                  clk_i,
    input  logic                  rst_ni,
    input  logic                  jtag_tck_i,
    input  logic                  jtag_tms_i,
    input  logic                  jtag_trst_ni,
    input  logic                  jtag_td_i,
    output logic                  jtag_td_o,
    input  logic [          31:0] boot_addr_i,
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
    input  logic [           1:0] i2c_scl_pad_i,
    output logic [           1:0] i2c_scl_pad_o,
    output logic [           1:0] i2c_scl_padoen_o,
    input  logic [           1:0] i2c_sda_pad_i,
    output logic [           1:0] i2c_sda_pad_o,
    output logic [           1:0] i2c_sda_padoen_o,
    output logic                  spi_clk_o,
    output logic [           1:0] spi_mode_o,
    output logic [           3:0] spi_csn_o,
    output logic [           3:0] spi_sdo_o,
    input  logic [           3:0] spi_sdi_i
);

  localparam int unsigned AddrWidth = 32;
  localparam int unsigned DataWidth = 32;

  localparam int unsigned NrIrqs = Cfg.irq.num_src;
  localparam int unsigned TGSize = Cfg.size_tg;

  // External, peripherals, mailbox
  localparam int unsigned NumAxiDemux = 3;
  localparam int unsigned AxiSelWidth = $clog2(NumAxiDemux);


  logic [NrIrqs-1:0] all_irqs;
  logic              mtime_irq;
  logic [       1:0] i2c_irq;
  logic [       1:0] spi_irq;
  logic [       1:0] mbx_irq;
  logic              uart_irq;

  logic [AxiSelWidth-1:0] demux_ar_sel, demux_aw_sel;

  // Timer queue intetrrupts aliased with timer group
  logic [(TGSize*2)-1:0] apb_timer_irqs;
  logic [(TGSize*2)-1:0] tq_irqs;

  logic                  tq_full;
  logic                  tq_nfull;

  logic [          63:0] mtime;
  logic                  intc_mtime_en;


  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_sbr ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_core ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_demux[NumAxiDemux-1:0] ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_mgr ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_mbx[1:0] ();

  AXI_LITE #(
      .AXI_ADDR_WIDTH(AddrWidth),
      .AXI_DATA_WIDTH(DataWidth)
  ) axi_tieoff ();

  zeroheti_core #(
      .Cfg(Cfg)
  ) i_core (
      .clk_i,
      .rst_ni,
      .testmode_i     (1'b0),
      .mtime_i        (mtime),
      .boot_addr_i,
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .intc_mtime_en_i(intc_mtime_en),
      .ext_irqs_i     (all_irqs),
      .ext_mgr        (axi_core),
      .ext_sbr        (axi_tieoff)
  );

  axi_lite_demux_intf #(
      .AxiAddrWidth(32'd32),
      .AxiDataWidth(32'd32),
      .NoMstPorts(NumAxiDemux),
      .MaxTrans(32'd1)
  ) i_demux (
      .clk_i,
      .rst_ni,
      .slv_aw_select_i(demux_aw_sel),
      .slv_ar_select_i(demux_ar_sel),
      .test_i         (1'b0),
      .slv            (axi_core),
      .mst            (axi_demux)
  );

  axi_lite_mailbox_intf #(
      .MAILBOX_DEPTH (32'd8),
      .IRQ_EDGE_TRIG (1'b1),
      .IRQ_ACT_HIGH  (1'b1),
      .AXI_ADDR_WIDTH(32'd32),
      .AXI_DATA_WIDTH(32'd32)
  ) i_mbx (
      .clk_i,
      .rst_ni,
      .test_i     (1'b0),
      .slv        (axi_mbx),
      .irq_o      (mbx_irq),
      .base_addr_i({Cfg.am.mbx.base, Cfg.am.mbx.base})
  );

  zeroheti_peripherals #(
      .Cfg   (Cfg),
      .TGSize(TGSize)
  ) i_peripherals (
      .clk_i,
      .rst_ni,
      .uart_rx_i,
      .uart_tx_o,
      .uart_irq_o (uart_irq),
      .mtime_en_o (intc_mtime_en),
      .mtime_irq_o(mtime_irq),
      .timer_irq_o(apb_timer_irqs),
      .tq_irq_o   (tq_irqs),
      .tq_full_o  (tq_full),
      .tq_nfull_o (tq_nfull),
      .i2c_irq_o  (i2c_irq),
      .i2c_scl_pad_i,
      .i2c_scl_pad_o,
      .i2c_scl_padoen_o,
      .i2c_sda_pad_i,
      .i2c_sda_pad_o,
      .i2c_sda_padoen_o,
      .spi_irq_o  (spi_irq),
      .spi_clk_o,
      .spi_csn_o,
      .spi_mode_o,
      .spi_sdi_i,
      .spi_sdo_o,
      .axi_sbr    (axi_demux[1])
  );

  axi_lite_join_intf i_join_ext (
      .in (axi_demux[2]),
      .out(axi_mgr)
  );

  axi_lite_join_intf i_join_mbx_core (
      .in (axi_demux[0]),
      .out(axi_mbx[0])
  );

  axi_lite_join_intf i_join_mbx_ext (
      .in (axi_sbr),
      .out(axi_mbx[1])
  );

  always_comb begin : demux_sel

    // Default: external memory
    demux_ar_sel = 2;
    demux_aw_sel = 2;

    unique case (axi_core.ar_addr) inside
      [Cfg.am.mbx.base : Cfg.am.mbx.last - 1]:       demux_ar_sel = 0;
      [Cfg.am.periph.base : Cfg.am.periph.last - 1]: demux_ar_sel = 1;
      default:                                       ;
    endcase

    unique case (axi_core.aw_addr) inside
      [Cfg.am.mbx.base : Cfg.am.mbx.last - 1]:       demux_aw_sel = 0;
      [Cfg.am.periph.base : Cfg.am.periph.last - 1]: demux_aw_sel = 1;
      default:                                       ;
    endcase
  end

  always_comb begin : irq_mapping
    all_irqs                       = '0;
    all_irqs[3]                    = 1'b0;  // legacy sw irq
    all_irqs[7]                    = mtime_irq;  // legacy tmr irq
    all_irqs[11]                   = 1'b0;  // legacy ext irq
    all_irqs[17:16]                = mbx_irq;
    all_irqs[19:18]                = spi_irq;
    all_irqs[21:20]                = i2c_irq;
    all_irqs[22]                   = tq_full;
    all_irqs[23]                   = tq_nfull;
    all_irqs[24]                   = uart_irq;
    //all_irqs[31]                 = nmi, reserved;
    all_irqs[((2*TGSize)+32)-1:32] = apb_timer_irqs | tq_irqs;
    all_irqs[NrIrqs-1:64]          = ext_irq_i;
  end : irq_mapping


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

  assign axi_tieoff.aw_addr  = 32'h0;
  assign axi_tieoff.aw_prot  = 3'h0;
  assign axi_tieoff.aw_valid = 1'b0;

  assign axi_tieoff.w_data   = 32'h0;
  assign axi_tieoff.w_strb   = 4'h0;
  assign axi_tieoff.w_valid  = 1'h0;

  assign axi_tieoff.b_ready  = 1'h0;

  assign axi_tieoff.ar_addr  = 32'h0;
  assign axi_tieoff.ar_prot  = 3'h0;
  assign axi_tieoff.ar_valid = 1'b0;
  
  assign axi_tieoff.r_ready  = 1'h0;


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

      // Preload 8 IMEM banks
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_0.hex"},
                  i_core.i_imem.g_banks[0].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_1.hex"},
                  i_core.i_imem.g_banks[1].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_2.hex"},
                  i_core.i_imem.g_banks[2].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_3.hex"},
                  i_core.i_imem.g_banks[3].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_4.hex"},
                  i_core.i_imem.g_banks[4].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_5.hex"},
                  i_core.i_imem.g_banks[5].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_6.hex"},
                  i_core.i_imem.g_banks[6].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/imem_7.hex"},
                  i_core.i_imem.g_banks[7].i_sram.sram);

      // Preload 4 DMEM banks
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_0.hex"},
                  i_core.i_dmem.g_banks[0].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_1.hex"},
                  i_core.i_dmem.g_banks[1].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_2.hex"},
                  i_core.i_dmem.g_banks[2].i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/stims/dmem_3.hex"},
                  i_core.i_dmem.g_banks[3].i_sram.sram);
    end
  end

`endif
`endif


endmodule : zeroheti_top

