module zeroheti_core_xbar #()(
  input  clk_i,
  input  rst_ni,
  OBI_BUS.Manager     cpu_if,
  OBI_BUS.Manager     cpu_lsu,
  OBI_BUS.Manager     sba_mgr,
  OBI_BUS.Subordinate dbg_sbr,
  OBI_BUS.Subordinate inst_sbr,
  OBI_BUS.Subordinate data_sbr,
  OBI_BUS.Subordinate apb_sbr
);
endmodule : zeroheti_core_xbar

