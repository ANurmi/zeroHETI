module zeroheti_peripherals #(
    parameter zeroheti_pkg::platform_cfg_t Cfg = zeroheti_pkg::EDFICCfg,
    parameter int unsigned TGSize = 8
) (
    input  logic                           clk_i,
    input  logic                           rst_ni,
    input  logic                           uart_rx_i,
    output logic                           uart_tx_o,
    output logic                           uart_irq_o,
    output logic                           mtime_en_o,
    output logic                           mtime_irq_o,
    output logic          [(TGSize*2)-1:0] timer_irq_o,
    output logic          [(TGSize*2)-1:0] tq_irq_o,
    output logic                           tq_full_o,
    output logic                           tq_nfull_o,
    output logic          [           1:0] i2c_irq_o,
    input  logic          [           1:0] i2c_scl_pad_i,
    output logic          [           1:0] i2c_scl_pad_o,
    output logic          [           1:0] i2c_scl_padoen_o,
    input  logic          [           1:0] i2c_sda_pad_i,
    output logic          [           1:0] i2c_sda_pad_o,
    output logic          [           1:0] i2c_sda_padoen_o,
    output logic          [           1:0] spi_irq_o,
    output logic                           spi_clk_o,
    output logic          [           3:0] spi_csn_o,
    output logic          [           1:0] spi_mode_o,
    output logic          [           3:0] spi_sdo_o,
    input  logic          [           3:0] spi_sdi_i,
           AXI_LITE.Slave                  axi_sbr
);

  localparam zeroheti_address_map_pkg::periph_map_t PerMap = Cfg.am.periph;
  localparam int unsigned NrApbPerip = PerMap.num;
  localparam int unsigned SelWidth = $clog2(NrApbPerip);
  localparam int unsigned ApbWidth = 32;

  localparam axi_pkg::xbar_rule_32_t [NrApbPerip-1:0] AddrMap = '{
      '{idx: 32'd7, start_addr: PerMap.spi.base, end_addr: PerMap.spi.last},
      '{idx: 32'd6, start_addr: PerMap.cfg.base, end_addr: PerMap.cfg.last},
      '{idx: 32'd5, start_addr: PerMap.tq.base, end_addr: PerMap.tq.last},
      '{idx: 32'd4, start_addr: PerMap.tg.base, end_addr: PerMap.tg.last},
      '{idx: 32'd3, start_addr: PerMap.i2c1.base, end_addr: PerMap.i2c1.last},
      '{idx: 32'd2, start_addr: PerMap.i2c0.base, end_addr: PerMap.i2c0.last},
      '{idx: 32'd1, start_addr: PerMap.mtimer.base, end_addr: PerMap.mtimer.last},
      '{idx: 32'd0, start_addr: PerMap.uart.base, end_addr: PerMap.uart.last}
  };

  logic [          63:0]       mtime;

  logic [          31:0]       paddr;
  logic [           2:0]       pprot;
  logic                        penable;
  logic [NrApbPerip-1:0]       psel;
  logic [           3:0]       pstrb;
  logic [          31:0]       pwdata;
  logic                        pwrite;
  logic [NrApbPerip-1:0][31:0] prdata;
  logic [NrApbPerip-1:0]       pready;
  logic [NrApbPerip-1:0]       pslverr;

  axi_lite_to_apb_intf #(
      .NoApbSlaves(NrApbPerip),
      .NoRules    (NrApbPerip),
      .rule_t     (axi_pkg::xbar_rule_32_t)
  ) i_axi_to_apb (
      .clk_i,
      .rst_ni,
      .slv       (axi_sbr),
      .paddr_o   (paddr),
      .pprot_o   (pprot),
      .pselx_o   (psel),
      .penable_o (penable),
      .pwrite_o  (pwrite),
      .pwdata_o  (pwdata),
      .pstrb_o   (pstrb),
      .pready_i  (pready),
      .prdata_i  (prdata),
      .pslverr_i (pslverr),
      .addr_map_i(AddrMap)
  );

  uart_wrapper #() i_uart (
      .clk_i,
      .rst_ni,
      .penable_i(penable),
      .pwrite_i (pwrite),
      .paddr_i  (paddr),
      .psel_i   (psel[0]),
      .pwdata_i (pwdata),
      .pstrb_i  (pstrb),
      .prdata_o (prdata[0]),
      .pready_o (pready[0]),
      .pslverr_o(pslverr[0]),
      .rx_i     (uart_rx_i),
      .tx_o     (uart_tx_o),
      .irq_o    (uart_irq_o)
  );

  apb_mtimer i_mtimer (
      .clk_i,
      .rst_ni,
      .penable_i  (penable),
      .pwrite_i   (pwrite),
      .paddr_i    (paddr),
      .psel_i     (psel[1]),
      .pwdata_i   (pwdata),
      .prdata_o   (prdata[1]),
      .pready_o   (pready[1]),
      .pslverr_o  (pslverr[1]),
      .mtime_o    (mtime),
      .timer_irq_o(mtime_irq_o)
  );

  apb_timer #(
      .APB_ADDR_WIDTH(ApbWidth),
      .TIMER_CNT(TGSize)
  ) i_apb_timer (
      .HCLK   (clk_i),
      .HRESETn(rst_ni),
      .PENABLE(penable),
      .PWRITE (pwrite),
      .PADDR  (paddr),
      .PSEL   (psel[4]),
      .PWDATA (pwdata),
      .PRDATA (prdata[4]),
      .PREADY (pready[4]),
      .PSLVERR(pslverr[4]),
      .irq_o  (timer_irq_o)
  );

  apb_i2c #(
      .APB_ADDR_WIDTH(32'd32)
  ) i_i2c_0 (
      .HCLK        (clk_i),
      .HRESETn     (rst_ni),
      .PADDR       (paddr),
      .PWDATA      (pwdata),
      .PWRITE      (pwrite),
      .PSEL        (psel[2]),
      .PENABLE     (penable),
      .PRDATA      (prdata[2]),
      .PREADY      (pready[2]),
      .PSLVERR     (pslverr[2]),
      .interrupt_o (i2c_irq_o[0]),
      .scl_pad_i   (i2c_scl_pad_i[0]),
      .scl_pad_o   (i2c_scl_pad_o[0]),
      .scl_padoen_o(i2c_scl_padoen_o[0]),
      .sda_pad_i   (i2c_sda_pad_i[0]),
      .sda_pad_o   (i2c_sda_pad_o[0]),
      .sda_padoen_o(i2c_sda_padoen_o[0])
  );

  apb_i2c #(
      .APB_ADDR_WIDTH(32'd32)
  ) i_i2c_1 (
      .HCLK        (clk_i),
      .HRESETn     (rst_ni),
      .PADDR       (paddr),
      .PWDATA      (pwdata),
      .PWRITE      (pwrite),
      .PSEL        (psel[3]),
      .PENABLE     (penable),
      .PRDATA      (prdata[3]),
      .PREADY      (pready[3]),
      .PSLVERR     (pslverr[3]),
      .interrupt_o (i2c_irq_o[1]),
      .scl_pad_i   (i2c_scl_pad_i[1]),
      .scl_pad_o   (i2c_scl_pad_o[1]),
      .scl_padoen_o(i2c_scl_padoen_o[1]),
      .sda_pad_i   (i2c_sda_pad_i[1]),
      .sda_pad_o   (i2c_sda_pad_o[1]),
      .sda_padoen_o(i2c_sda_padoen_o[1])
  );

  apb_cfg_regs #(
      .Cfg(Cfg)
  ) i_cfg_regs (
      .clk_i,
      .rst_ni,
      .intc_mtime_en_o(mtime_en_o),
      .penable_i(penable),
      .pwrite_i (pwrite),
      .paddr_i  (paddr),
      .psel_i   (psel[6]),
      .pwdata_i (pwdata),
      .pstrb_i  (pstrb),
      .prdata_o (prdata[6]),
      .pready_o (pready[6]),
      .pslverr_o(pslverr[6])
  );

  apb_spi_master #(
      .APB_ADDR_WIDTH(ApbWidth)
  ) i_apb_spi (
      .HCLK    (clk_i),
      .HRESETn (rst_ni),
      .PENABLE (penable),
      .PWRITE  (pwrite),
      .PADDR   (paddr),
      .PSEL    (psel[7]),
      .PWDATA  (pwdata),
      .PRDATA  (prdata[7]),
      .PREADY  (pready[7]),
      .PSLVERR (pslverr[7]),
      .events_o(spi_irq_o),
      .spi_clk (spi_clk_o),
      .spi_csn0(spi_csn_o[0]),
      .spi_csn1(spi_csn_o[1]),
      .spi_csn2(spi_csn_o[2]),
      .spi_csn3(spi_csn_o[3]),
      .spi_mode(spi_mode_o),
      .spi_sdo0(spi_sdo_o[0]),
      .spi_sdo1(spi_sdo_o[1]),
      .spi_sdo2(spi_sdo_o[2]),
      .spi_sdo3(spi_sdo_o[3]),
      .spi_sdi0(spi_sdi_i[0]),
      .spi_sdi1(spi_sdi_i[1]),
      .spi_sdi2(spi_sdi_i[2]),
      .spi_sdi3(spi_sdi_i[3])
  );

  apb_timer_queue #(
      .NrIrqs(TGSize * 2),
      .Depth (TGSize * 2)
  ) i_tq (
      .clk_i,
      .rst_ni,
      .mtime_i    (mtime),
      .irq_pl_o   (tq_irq_o),
      .irq_full_o (tq_full_o),
      .irq_nfull_o(tq_nfull_o),
      .penable_i  (penable),
      .pwrite_i   (pwrite),
      .paddr_i    (paddr),
      .psel_i     (psel[5]),
      .pwdata_i   (pwdata),
      .pstrb_i    (pstrb),
      .prdata_o   (prdata[5]),
      .pready_o   (pready[5]),
      .pslverr_o  (pslverr[5])
  );


endmodule : zeroheti_peripherals

