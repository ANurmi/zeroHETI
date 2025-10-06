module zeroheti_top import zeroheti_pkg::AddrMap; #()(
  input  logic clk_i,
  input  logic rst_ni,
  input  logic jtag_tck_i,
  input  logic jtag_tms_i,
  input  logic jtag_trst_ni,
  input  logic jtag_td_i,
  output logic jtag_td_o,
  input  logic uart_rx_i,
  output logic uart_tx_o
);

localparam zeroheti_pkg::core_cfg_t CoreCfg = zeroheti_pkg::DefaultCfg;

localparam int unsigned ApbWidth   = 32;
localparam int unsigned DataWidth  = 32;
localparam int unsigned NrApbPerip = 4;
localparam int unsigned SelWidth   = $clog2(NrApbPerip);

APB core_apb ();
APB demux_apb [NrApbPerip]();

logic [SelWidth-1:0] demux_sel;

zeroheti_core #(
  .Cfg (CoreCfg)
) i_core (
  .clk_i,
  .rst_ni,
  .testmode_i (1'b0),
  .jtag_tck_i,
  .jtag_tms_i,
  .jtag_trst_ni,
  .jtag_td_i,
  .jtag_td_o,
  .apb_mgr     (core_apb)
);

always_comb begin : apb_decode
  unique case (core_apb.paddr) inside
  //[AddrMap.hetic.base :AddrMap.hetic.last-1 ]: demux_sel = SelWidth'('d3);
  [AddrMap.uart.base  :AddrMap.uart.last-1  ]: demux_sel = SelWidth'('d2);
  [AddrMap.mtimer.base:AddrMap.mtimer.last-1]: demux_sel = SelWidth'('d1);
  default: demux_sel = SelWidth'('d0);
  endcase
end

apb_demux_intf #(
  .APB_ADDR_WIDTH (ApbWidth),
  .APB_DATA_WIDTH (DataWidth),
  .NoMstPorts     (NrApbPerip)
) i_apb_demux (
  .slv      (core_apb),
  .mst      (demux_apb),
  .select_i (demux_sel)
);

/*
apb_hetic #(
  .NrIrqLines (CoreCfg.num_irqs),
  .NrIrqPrios (CoreCfg.num_prio)
) i_hetic (
  .clk_i,
  .rst_ni,
  .penable_i  (demux_apb[0].penable),
  .pwrite_i   (demux_apb[0].pwrite),
  .paddr_i    (demux_apb[0].paddr),
  .psel_i     (demux_apb[0].psel),
  .pwdata_i   (demux_apb[0].pwdata),
  .prdata_o   (demux_apb[0].prdata),
  .pready_o   (demux_apb[0].pready),
  .pslverr_o  (demux_apb[0].pslverr),
  .irq_heti_o (irq_heti),
  .irq_nest_o (/*not used yet),
  .irq_o      (irq),
  .irq_id_i   (irq_id),
  .irq_ack_i  (irq_ack)
);
*/


`ifdef VERILATOR
mock_uart i_mock_uart (
  .clk_i,
  .rst_ni,
  .penable_i (demux_apb[1].penable),
  .pwrite_i  (demux_apb[1].pwrite),
  .paddr_i   (demux_apb[1].paddr),
  .psel_i    (demux_apb[1].psel),
  .pwdata_i  (demux_apb[1].pwdata),
  .prdata_o  (demux_apb[1].prdata),
  .pready_o  (demux_apb[1].pready),
  .pslverr_o (demux_apb[1].pslverr)
);
`else

apb_uart i_apb_uart (
  .CLK      (clk_i),
  .RSTN     (rst_ni),
  .PSEL     (demux_apb[1].psel),
  .PENABLE  (demux_apb[1].penable),
  .PWRITE   (demux_apb[1].pwrite),
  .PADDR    (demux_apb[1].paddr[4:2]),
  .PWDATA   (demux_apb[1].pwdata),
  .PRDATA   (demux_apb[1].prdata),
  .PREADY   (demux_apb[1].pready),
  .PSLVERR  (demux_apb[1].pslverr),
  .INT      (),
  .CTSN     (1'b0),
  .DSRN     (1'b0),
  .DCDN     (1'b0),
  .RIN      (1'b0),
  .RTSN     (),
  .OUT1N    (),
  .OUT2N    (),
  .DTRN     (),
  .SIN      (uart_rx_i),
  .SOUT     (uart_tx_o)
);

`endif

apb_mtimer i_mtimer (
  .clk_i,
  .rst_ni,
  .penable_i   (demux_apb[2].penable),
  .pwrite_i    (demux_apb[2].pwrite),
  .paddr_i     (demux_apb[2].paddr),
  .psel_i      (demux_apb[2].psel),
  .pwdata_i    (demux_apb[2].pwdata),
  .prdata_o    (demux_apb[2].prdata),
  .pready_o    (demux_apb[2].pready),
  .pslverr_o   (demux_apb[2].pslverr),
  .timer_irq_o ()
);

//assign demux_apb[0].pready = 1'b1;
//assign demux_apb[1].pready = 1'b1;
//assign demux_apb[2].pready = 1'b1;
assign demux_apb[3].pready = 1'b1;

`ifndef SYNTHESIS

typedef enum bit {JTAG, READMEM} load_e;
load_e LoadType;

initial begin : simulation_loader

  LoadType = `LOAD;

  if (LoadType == READMEM) begin
    @(posedge rst_ni);
    $display("Initializing program with $readmemh");
    $display("APPLICABLE TO SIMULATED DESIGNS ONLY");
    $readmemh("../build/verilator_build/imem_stim.hex", i_core.i_imem.i_sram.sram);
    $readmemh("../build/verilator_build/dmem_stim.hex", i_core.i_dmem.i_sram.sram);
  end
end

`endif


endmodule : zeroheti_top

