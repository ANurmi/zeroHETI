`include "axi/typedef.svh"
`include "axi/assign.svh"

/// Interface variant of `axi_lite_from_mem` (upstream `axi` provides only the struct version).
/// See `axi_lite_from_mem` for parameter and port documentation.
module axi_lite_from_mem_intf #(
    /// Memory request address width.
    parameter int unsigned    MEM_ADDR_WIDTH = 32'd0,
    /// AXI4-Lite address width.
    parameter int unsigned    AXI_ADDR_WIDTH = 32'd0,
    /// Data width in bits, shared by memory and AXI4-Lite.
    parameter int unsigned    DATA_WIDTH     = 32'd0,
    /// Maximum number of outstanding requests.
    parameter int unsigned    MAX_REQUESTS   = 32'd0,
    /// Protection signal driven on all AXI4-Lite requests.
    parameter axi_pkg::prot_t AXI_PROT       = 3'b000,
    /// Dependent parameters, do not override.
    localparam type           mem_addr_t     = logic [MEM_ADDR_WIDTH-1:0],
    localparam type           data_t         = logic [DATA_WIDTH-1:0],
    localparam type           strb_t         = logic [DATA_WIDTH/8-1:0]
) (
    input  logic          clk_i,
    input  logic          rst_ni,
    input  logic          mem_req_i,
    input  mem_addr_t     mem_addr_i,
    input  logic          mem_we_i,
    input  data_t         mem_wdata_i,
    input  strb_t         mem_be_i,
    output logic          mem_gnt_o,
    output logic          mem_rsp_valid_o,
    output data_t         mem_rsp_rdata_o,
    output logic          mem_rsp_error_o,
           AXI_LITE.Master mst
);

  typedef logic [AXI_ADDR_WIDTH-1:0] axi_addr_t;

  `AXI_LITE_TYPEDEF_ALL(axi_lite, axi_addr_t, data_t, strb_t)

  axi_lite_req_t  axi_req;
  axi_lite_resp_t axi_rsp;

  axi_lite_from_mem #(
      .MemAddrWidth(MEM_ADDR_WIDTH),
      .AxiAddrWidth(AXI_ADDR_WIDTH),
      .DataWidth   (DATA_WIDTH),
      .MaxRequests (MAX_REQUESTS),
      .AxiProt     (AXI_PROT),
      .axi_req_t   (axi_lite_req_t),
      .axi_rsp_t   (axi_lite_resp_t)
  ) i_axi_lite_from_mem (
      .clk_i,
      .rst_ni,
      .mem_req_i,
      .mem_addr_i,
      .mem_we_i,
      .mem_wdata_i,
      .mem_be_i,
      .mem_gnt_o,
      .mem_rsp_valid_o,
      .mem_rsp_rdata_o,
      .mem_rsp_error_o,
      .axi_req_o(axi_req),
      .axi_rsp_i(axi_rsp)
  );

  `AXI_LITE_ASSIGN_FROM_REQ(mst, axi_req)
  `AXI_LITE_ASSIGN_TO_RESP(axi_rsp, mst)

// pragma translate_off
`ifndef VERILATOR
  initial begin : p_assertions
    assert (mst.AXI_ADDR_WIDTH == AXI_ADDR_WIDTH)
    else $fatal(1, "AXI_LITE interface address width does not match AXI_ADDR_WIDTH");
    assert (mst.AXI_DATA_WIDTH == DATA_WIDTH)
    else $fatal(1, "AXI_LITE interface data width does not match DATA_WIDTH");
  end
`endif
// pragma translate_on

endmodule : axi_lite_from_mem_intf
