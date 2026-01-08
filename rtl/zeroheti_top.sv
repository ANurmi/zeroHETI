module zeroheti_top
  import zeroheti_pkg::AddrMap;
  import zeroheti_pkg::TGSize;
#(
    parameter zeroheti_pkg::core_cfg_t CoreCfg = zeroheti_pkg::`CORE_CFG,
    localparam int unsigned NumIntIrqs = 16,
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
  localparam int unsigned NrApbPerip = 4;
  localparam int unsigned SelWidth = $clog2(NrApbPerip);

  APB core_apb ();
  APB demux_apb[NrApbPerip] ();

  logic [  SelWidth-1:0] demux_sel;
  logic [    NrIrqs-1:0] all_irqs;
  logic                  mtime_irq;
  logic                  i2c_irq;
  logic                  uart_irq;
  logic [(TGSize*2)-1:0] apb_timer_irqs;

  always_comb begin : irq_mapping
    all_irqs                      = '0;
    all_irqs[5]                   = uart_irq;
    all_irqs[6]                   = i2c_irq;
    all_irqs[7]                   = mtime_irq;
    all_irqs[((2*TGSize)+8)-1:8]  = apb_timer_irqs;
    all_irqs[NrIrqs-1:NumIntIrqs] = ext_irq_i;
  end : irq_mapping

  zeroheti_core #(
      .Cfg(CoreCfg)
  ) i_core (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .ext_irqs_i(all_irqs),
      .apb_mgr   (core_apb)
  );

  always_comb begin : apb_decode
    unique case (core_apb.paddr) inside
      [AddrMap.tg.base : AddrMap.tg.last - 1]:         demux_sel = SelWidth'('d3);
      [AddrMap.uart.base : AddrMap.uart.last - 1]:     demux_sel = SelWidth'('d2);
      [AddrMap.mtimer.base : AddrMap.mtimer.last - 1]: demux_sel = SelWidth'('d1);
      [AddrMap.i2c.base : AddrMap.i2c.last - 1]:       demux_sel = SelWidth'('d0);
      default: begin
        demux_sel = SelWidth'('d0);
        if (core_apb.psel & core_apb.penable) $display("Warning: APB access to unmapped region!");
      end
    endcase
  end

  apb_demux_intf #(
      .APB_ADDR_WIDTH(ApbWidth),
      .APB_DATA_WIDTH(DataWidth),
      .NoMstPorts    (NrApbPerip)
  ) i_apb_demux (
      .slv     (core_apb),
      .mst     (demux_apb),
      .select_i(demux_sel)
  );


`ifndef FULL_UART
  mock_uart i_mock_uart (
      .clk_i,
      .rst_ni,
      .penable_i(demux_apb[1].penable),
      .pwrite_i (demux_apb[1].pwrite),
      .paddr_i  (demux_apb[1].paddr),
      .psel_i   (demux_apb[1].psel),
      .pwdata_i (demux_apb[1].pwdata),
      .prdata_o (demux_apb[1].prdata),
      .pready_o (demux_apb[1].pready),
      .pslverr_o(demux_apb[1].pslverr)
  );
  assign uart_irq = 1'b0;
`else
  apb_uart i_apb_uart (
      .CLK    (clk_i),
      .RSTN   (rst_ni),
      .PSEL   (demux_apb[1].psel),
      .PENABLE(demux_apb[1].penable),
      .PWRITE (demux_apb[1].pwrite),
      .PADDR  (demux_apb[1].paddr[4:2]),
      .PWDATA (demux_apb[1].pwdata),
      .PRDATA (demux_apb[1].prdata),
      .PREADY (demux_apb[1].pready),
      .PSLVERR(demux_apb[1].pslverr),
      .INT    (uart_irq),
      .CTSN   (1'b0),
      .DSRN   (1'b0),
      .DCDN   (1'b0),
      .RIN    (1'b0),
      .RTSN   (),
      .OUT1N  (),
      .OUT2N  (),
      .DTRN   (),
      .SIN    (uart_rx_i),
      .SOUT   (uart_tx_o)
  );

`endif

  apb_mtimer i_mtimer (
      .clk_i,
      .rst_ni,
      .penable_i  (demux_apb[2].penable),
      .pwrite_i   (demux_apb[2].pwrite),
      .paddr_i    (demux_apb[2].paddr),
      .psel_i     (demux_apb[2].psel),
      .pwdata_i   (demux_apb[2].pwdata),
      .prdata_o   (demux_apb[2].prdata),
      .pready_o   (demux_apb[2].pready),
      .pslverr_o  (demux_apb[2].pslverr),
      .timer_irq_o(mtime_irq)
  );

  apb_timer #(
      .APB_ADDR_WIDTH(ApbWidth),
      .TIMER_CNT(TGSize)
  ) i_apb_timer (
      .HCLK   (clk_i),
      .HRESETn(rst_ni),
      .PENABLE(demux_apb[0].penable),
      .PWRITE (demux_apb[0].pwrite),
      .PADDR  (demux_apb[0].paddr),
      .PSEL   (demux_apb[0].psel),
      .PWDATA (demux_apb[0].pwdata),
      .PRDATA (demux_apb[0].prdata),
      .PREADY (demux_apb[0].pready),
      .PSLVERR(demux_apb[0].pslverr),
      .irq_o  (apb_timer_irqs)
  );

  apb_i2c #(
      .APB_ADDR_WIDTH(32'd32)
  ) i_i2c (
      .HCLK        (clk_i),
      .HRESETn     (rst_ni),
      .PADDR       (demux_apb[3].paddr),
      .PWDATA      (demux_apb[3].pwdata),
      .PWRITE      (demux_apb[3].pwrite),
      .PSEL        (demux_apb[3].psel),
      .PENABLE     (demux_apb[3].penable),
      .PRDATA      (demux_apb[3].prdata),
      .PREADY      (demux_apb[3].pready),
      .PSLVERR     (demux_apb[3].pslverr),
      .interrupt_o (i2c_irq),
      .scl_pad_i   (i2c_scl_pad_i),
      .scl_pad_o   (i2c_scl_pad_o),
      .scl_padoen_o(i2c_scl_padoen_o),
      .sda_pad_i   (i2c_sda_pad_i),
      .sda_pad_o   (i2c_sda_pad_o),
      .sda_padoen_o(i2c_sda_padoen_o)
  );

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
      $readmemh({zeroHetiRoot, "/build/verilator_build/imem_stim.hex"}, i_core.i_imem.i_sram.sram);
      $readmemh({zeroHetiRoot, "/build/verilator_build/dmem_stim.hex"}, i_core.i_dmem.i_sram.sram);
    end
  end

`endif
`endif


endmodule : zeroheti_top

