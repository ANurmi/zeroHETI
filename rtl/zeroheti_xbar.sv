module zeroheti_xbar
  import zeroheti_pkg::AddrMap;
#(
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Subordinate inst_bus,
          OBI_BUS.Subordinate data_bus,
          OBI_BUS.Manager     imem_bus,
          OBI_BUS.Manager     dmem_bus,
          OBI_BUS.Manager     intc_bus,
          OBI_BUS.Manager     per_bus,
          OBI_BUS.Subordinate sba_bus,
          OBI_BUS.Manager     dbg_bus,
          OBI_BUS.Manager     mbx_bus
);

  localparam int unsigned DemuxMaxTrans = 32'd1;
  localparam int unsigned MuxMaxTrans = 32'd1;

  localparam int unsigned MuxInstInputs = 2;
  localparam int unsigned MuxDataInputs = 2;
  localparam int unsigned MuxIntcInputs = 2;
  localparam int unsigned MuxPerInputs = 2;
  localparam int unsigned MuxDbgInputs = 3;
  localparam int unsigned MuxMbxInputs = 2;

  localparam int unsigned SbaAccess = 32'd6;
  localparam int unsigned InstAccess = 32'd2;
  localparam int unsigned Dataccess = 32'd5;

  OBI_BUS sba_demux[SbaAccess] ();
  OBI_BUS inst_demux[InstAccess] ();
  OBI_BUS data_demux[Dataccess] ();

  OBI_BUS imem_mux[MuxInstInputs] ();
  OBI_BUS dmem_mux[MuxDataInputs] ();
  OBI_BUS intc_mux[MuxIntcInputs] ();
  OBI_BUS per_mux[MuxPerInputs] ();
  OBI_BUS dbg_mux[MuxDbgInputs] ();
  OBI_BUS mbx_mux[MuxMbxInputs] ();

  logic [2:0] sba_sel;
  logic       inst_sel;
  logic [2:0] data_sel;

  assign inst_sel = !((inst_bus.addr >= AddrMap.imem.base) & (inst_bus.addr < AddrMap.dmem.base));

  always_comb begin : g_sba_decode
    sba_sel = 0;
    unique case (sba_bus.addr) inside
      [AddrMap.dbg.base : AddrMap.dbg.last]:     sba_sel = 0;
      [AddrMap.imem.base : AddrMap.imem.last]:   sba_sel = 1;
      [AddrMap.dmem.base : AddrMap.dmem.last]:   sba_sel = 2;
      [AddrMap.hetic.base : AddrMap.hetic.last]: sba_sel = 3;
      [AddrMap.uart.base : AddrMap.i2c.last]:    sba_sel = 4;
      [AddrMap.ext.base : AddrMap.ext.last]:     sba_sel = 5;
      default:                                   ;
    endcase
  end : g_sba_decode

  always_comb begin : g_data_decode
    data_sel = 0;
    unique case (data_bus.addr) inside
      [AddrMap.dbg.base : AddrMap.dbg.last]:     data_sel = 0;
      [AddrMap.dmem.base : AddrMap.dmem.last]:   data_sel = 1;
      [AddrMap.hetic.base : AddrMap.hetic.last]: data_sel = 2;
      [AddrMap.uart.base : AddrMap.i2c.last]:    data_sel = 3;
      [AddrMap.ext.base : AddrMap.ext.last]:     data_sel = 4;
      default:                                   ;
    endcase
  end : g_data_decode

  // Manager demuxes
  obi_demux_intf #(
      .NumMgrPorts(SbaAccess),
      .NumMaxTrans(DemuxMaxTrans)
  ) i_sba_demux (
      .clk_i,
      .rst_ni,
      .sbr_port_select_i(sba_sel),
      .sbr_port(sba_bus),
      .mgr_ports(sba_demux)
  );
  obi_demux_intf #(
      .NumMgrPorts(InstAccess),
      .NumMaxTrans(DemuxMaxTrans)
  ) i_inst_demux (
      .clk_i,
      .rst_ni,
      .sbr_port_select_i(inst_sel),
      .sbr_port(inst_bus),
      .mgr_ports(inst_demux)
  );
  obi_demux_intf #(
      .NumMgrPorts(Dataccess),
      .NumMaxTrans(DemuxMaxTrans)
  ) i_data_demux (
      .clk_i,
      .rst_ni,
      .sbr_port_select_i(data_sel),
      .sbr_port(data_bus),
      .mgr_ports(data_demux)
  );

  // Internal routing w/ potential cuts

  // SBA routing
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_dbg (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[0]),
      .obi_m(dbg_mux[0])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_imem (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[1]),
      .obi_m(imem_mux[0])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_dmem (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[2]),
      .obi_m(dmem_mux[0])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_intc (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[3]),
      .obi_m(intc_mux[0])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_per (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[4]),
      .obi_m(per_mux[0])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_sba_mbx (
      .clk_i,
      .rst_ni,
      .obi_s(sba_demux[5]),
      .obi_m(mbx_mux[0])
  );

  // Instruction fetch routing
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_inst_imem (
      .clk_i,
      .rst_ni,
      .obi_s(inst_demux[0]),
      .obi_m(imem_mux[1])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_inst_dbg (
      .clk_i,
      .rst_ni,
      .obi_s(inst_demux[1]),
      .obi_m(dbg_mux[1])
  );

  // Load-store unit routing
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_data_dbg (
      .clk_i,
      .rst_ni,
      .obi_s(data_demux[0]),
      .obi_m(dbg_mux[2])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_data_dmem (
      .clk_i,
      .rst_ni,
      .obi_s(data_demux[1]),
      .obi_m(dmem_mux[1])
  );
  obi_connection #(
      .Cut(1'b1)
  ) i_conn_data_intc (
      .clk_i,
      .rst_ni,
      .obi_s(data_demux[2]),
      .obi_m(intc_mux[1])
  );
  obi_connection #(
      .Cut(1'b1)
  ) i_conn_data_per (
      .clk_i,
      .rst_ni,
      .obi_s(data_demux[3]),
      .obi_m(per_mux[1])
  );
  obi_connection #(
      .Cut(1'b0)
  ) i_conn_data_mbx (
      .clk_i,
      .rst_ni,
      .obi_s(data_demux[4]),
      .obi_m(mbx_mux[1])
  );

  // Subordinate muxes
  obi_mux_intf #(
      .NumSbrPorts(MuxInstInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_imem_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (imem_mux),
      .mgr_port  (imem_bus)
  );
  obi_mux_intf #(
      .NumSbrPorts(MuxDataInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_dmem_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (dmem_mux),
      .mgr_port  (dmem_bus)
  );
  obi_mux_intf #(
      .NumSbrPorts(MuxIntcInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_intc_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (intc_mux),
      .mgr_port  (intc_bus)
  );
  obi_mux_intf #(
      .NumSbrPorts(MuxPerInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_per_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (per_mux),
      .mgr_port  (per_bus)
  );
  obi_mux_intf #(
      .NumSbrPorts(MuxDbgInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_dbg_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (dbg_mux),
      .mgr_port  (dbg_bus)
  );
  obi_mux_intf #(
      .NumSbrPorts(MuxMbxInputs),
      .NumMaxTrans(MuxMaxTrans)
  ) i_mbx_mux (
      .clk_i,
      .rst_ni,
      .testmode_i(1'b0),
      .sbr_ports (mbx_mux),
      .mgr_port  (mbx_bus)
  );

endmodule : zeroheti_xbar

