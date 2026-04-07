module vip_sim_env #(
    parameter type i2c_req_t = logic,
    parameter type i2c_rsp_t = logic
) (
    input  logic     clk_i,
    input  logic     rst_ni,
    input  i2c_req_t i2c_req_i,
    output i2c_rsp_t i2c_rsp_o
);

  typedef logic [31:0] dtype;
  typedef logic [6:0] atype;

  // use associative arrays for sim env memory space
  dtype array[atype];

  initial begin
    array = '{7'h68 : 32'hBA1155AB, 7'h11 : 32'hb0110c55, 7'h0: 32'hDEADBEEF};
  end

  assign i2c_rsp_o.rdata = array[i2c_req_i.addr];

  always @(posedge i2c_req_i.valid) begin
    if (i2c_req_i.write) begin
      $display("[VIP_I2C] write - addr: %h, data: %h", i2c_req_i.addr, i2c_req_i.wdata);
    end else begin
      $display("[VIP_I2C] read  - addr: %h, data: %h", i2c_req_i.addr, array[i2c_req_i.addr]);
    end
  end

  task automatic recv_letter(input logic [31:0] addr, input logic [31:0] data);
    $display("Dingdong: %h, %h", addr, data);
    if (addr == 32'h0400_0000) begin
      i_mbx_drv.send_letter(32'h1234_0000, 32'h0000_2222);
      i_mbx_drv.send_letter(32'h6767_6767, 32'h1234_1234);
      i_mbx_drv.raise_irq();
    end
  endtask

endmodule : vip_sim_env
