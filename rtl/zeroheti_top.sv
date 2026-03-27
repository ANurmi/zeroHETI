module zeroheti_top
  import zeroheti_pkg::AddrMap;
  import zeroheti_pkg::TGSize;
#(
    parameter zeroheti_pkg::core_cfg_t CoreCfg = zeroheti_pkg::`CORE_CFG,
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

  OBI_BUS mbx_obi ();
  APB core_apb ();
  APB demux_apb[NrApbPerip] ();

  logic [  SelWidth-1:0] demux_sel;
  logic [    NrIrqs-1:0] all_irqs;
  logic                  mtime_irq;
  logic                  i2c_irq;
  logic                  mbx_irq;
  logic                  uart_irq;
  logic [(TGSize*2)-1:0] apb_timer_irqs;

  logic [          63:0] mtime;

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
      .testmode_i(1'b0),
      .mtime_i   (mtime),
      .jtag_tck_i,
      .jtag_tms_i,
      .jtag_trst_ni,
      .jtag_td_i,
      .jtag_td_o,
      .ext_irqs_i(all_irqs),
      .obi_mgr   (mbx_obi),
      .apb_mgr   (core_apb)
  );

  obi_mbx i_mbx (
      .clk_i,
      .rst_ni,
      .irq_o  (mbx_irq),
      .obi_sbr(mbx_obi)
  );

  always_comb begin : apb_decode
    unique case (core_apb.paddr) inside
      [AddrMap.cfg.base : AddrMap.cfg.last - 1]:       demux_sel = SelWidth'('d0);
      [AddrMap.tg.base : AddrMap.tg.last - 1]:         demux_sel = SelWidth'('d1);
      [AddrMap.uart.base : AddrMap.uart.last - 1]:     demux_sel = SelWidth'('d2);
      [AddrMap.mtimer.base : AddrMap.mtimer.last - 1]: demux_sel = SelWidth'('d3);
      [AddrMap.i2c.base : AddrMap.i2c.last - 1]:       demux_sel = SelWidth'('d4);
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
      .penable_i(demux_apb[2].penable),
      .pwrite_i (demux_apb[2].pwrite),
      .paddr_i  (demux_apb[2].paddr),
      .psel_i   (demux_apb[2].psel),
      .pwdata_i (demux_apb[2].pwdata),
      .pstrb_i  (demux_apb[2].pstrb),
      .prdata_o (demux_apb[2].prdata),
      .pready_o (demux_apb[2].pready),
      .pslverr_o(demux_apb[2].pslverr)
  );
  assign uart_irq  = 1'b0;
  assign uart_tx_o = 1'b0;
`else
  logic [31:0] rdata_local;
  logic [31:0] wdata_local;
  logic [ 2:0] addr_local;
  logic [ 2:0] addr_offs;

  assign addr_local = demux_apb[2].paddr[2:0] + addr_offs;

  always_comb begin
    addr_offs           = 0;
    wdata_local         = 0;
    demux_apb[1].prdata = 0;
    unique case (demux_apb[2].pstrb)
      4'b0001: begin
        addr_offs = 0;
        demux_apb[2].prdata = {24'h0, rdata_local[7:0]};
        wdata_local = {24'h0, demux_apb[2].pwdata[7:0]};
      end
      4'b0010: begin
        addr_offs = 1;
        demux_apb[2].prdata = {16'h0, rdata_local[7:0], 8'h0};
        wdata_local = {24'h0, demux_apb[2].pwdata[15:8]};
      end
      4'b0100: begin
        addr_offs = 2;
        demux_apb[2].prdata = {8'h0, rdata_local[7:0], 16'h0};
        wdata_local = {24'h0, demux_apb[2].pwdata[23:16]};
      end
      4'b1000: begin
        addr_offs = 3;
        demux_apb[2].prdata = {rdata_local[7:0], 24'h0};
        wdata_local = {24'h0, demux_apb[2].pwdata[31:24]};
      end
      default: ;
    endcase
  end

  apb_uart i_apb_uart (
      .CLK    (clk_i),
      .RSTN   (rst_ni),
      .PSEL   (demux_apb[2].psel),
      .PENABLE(demux_apb[2].penable),
      .PWRITE (demux_apb[2].pwrite),
      .PADDR  (addr_local),
      .PWDATA (wdata_local),
      .PRDATA (rdata_local),
      .PREADY (demux_apb[2].pready),
      .PSLVERR(demux_apb[2].pslverr),
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
      .apb_i(demux_apb[4])
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

