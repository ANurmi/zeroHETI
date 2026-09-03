module obi_to_axi_lite_intf #(
    parameter int unsigned AxiAddrWidth = 0,
    parameter int unsigned AxiDataWidth = 0,
    parameter int unsigned AxiUserWidth = 0
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Subordinate obi_sbr,
          AXI_LITE.Master     axi_mgr
);

// TODO: implement

endmodule
