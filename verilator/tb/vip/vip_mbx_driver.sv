module vip_mbx_driver #(
    parameter type mbx_req_t = logic,
    parameter type mbx_rsp_t = logic
) (
    input logic           clk_i,
    input logic           rst_ni,
    input mbx_req_t       mbx_req_o,
    input mbx_rsp_t       mbx_rsp_i,
          AXI_LITE.Master axi_mgr
);
endmodule : vip_mbx_driver
