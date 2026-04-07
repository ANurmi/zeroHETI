module vip_sim_env #(
    parameter type i2c_req_t = logic,
    parameter type i2c_rsp_t = logic,
    parameter type mbx_req_t = logic,
    parameter type mbx_rsp_t = logic
) (
    input  logic     clk_i,
    input  logic     rst_ni,
    input  i2c_req_t i2c_req_i,
    output i2c_rsp_t i2c_rsp_o,
    output mbx_req_t mbx_req_o,
    input  mbx_rsp_t mbx_rsp_i
);

  typedef logic [31:0] dtype;
  typedef logic [6:0] atype;

  dtype array[atype];

  initial begin
    array = '{7'h68 : 32'hBA1155AB, 7'h11 : 32'hb0110c55, 7'h0: 32'hDEADBEEF};
  end

  assign i2c_rsp_o.rdata = array[i2c_req_i.addr];

  always @(posedge i2c_req_i.valid) begin
    if (i2c_req_i.write) begin
      $display("[VIP] write - addr: %h, data: %h", i2c_req_i.addr, i2c_req_i.wdata);
    end else begin
      $display("[VIP] read  - addr: %h, data: %h", i2c_req_i.addr, array[i2c_req_i.addr]);
    end
  end

  /*
  always @(posedge i2c_req_i.valid) begin
    $display("Got addr %h", i2c_req_i.addr);
    if (i2c_req_i.write) $display("Wdata: %h", i2c_req_i.wdata);
    else begin
      if (i2c_req_i.addr == 7'h68) i2c_rsp_o.rdata = 32'hAB;
      else i2c_rsp_o.rdata = 32'h11;
    end
    //i2c_rsp_o.valid = 1'b1;
    //@(i2c_req_i.valid == 0);
    //i2c_rsp_o.valid = 1'b0;
    //i2c_rsp_o.rdata = 32'h0;
  end
*/
endmodule : vip_sim_env
