module vip_sim_env #(
    parameter type i2c_req_t = logic,
    parameter type i2c_rsp_t = logic
) (
    input  logic     clk_i,
    input  logic     rst_ni,
    input  i2c_req_t i2c_req_i,
    output i2c_rsp_t i2c_rsp_o
);

  localparam int unsigned NrMotors = 4;

  localparam logic [31:0] SimStartAddr = 32'h0100_0000;
  localparam logic [31:0] SimEndAddr = 32'h0100_0001;

  logic        [3:0] motor_irqs;
  logic              motor_enable;

  int unsigned       motor_prescaler = 0;

  typedef logic [31:0] dtype;
  typedef logic [6:0] atype;

  // use associative arrays for i2c memory space
  dtype array[atype];

  initial begin
    array = '{
        7'h68 : 32'hBA11_55AB,
        7'h13 : 32'h0000_1234,
        7'h11 : 32'hb011_0c55,
        7'h0  : 32'hDEAD_BEEF
    };
  end

  logic sim_term_signal;
  assign sim_term_signal = i_zeroheti.i_core.dbg_bus.we
                         & i_zeroheti.i_core.dbg_bus.req
                         & i_zeroheti.i_core.dbg_bus.wdata[31]
                         & (i_zeroheti.i_core.dbg_bus.addr == 32'h0380);

  assign i2c_rsp_o.rdata = array[i2c_req_i.addr];

  always @(posedge i2c_req_i.valid) begin
    if (i2c_req_i.write) begin
      $display("[VIP_I2C] write - addr: %h, data: %h", i2c_req_i.addr, i2c_req_i.wdata);
    end else begin
      $display("[VIP_I2C] read  - addr: %h, data: %h", i2c_req_i.addr, array[i2c_req_i.addr]);
    end
  end

  always @(posedge sim_term_signal) begin
    i_vip_zeroheti_top.i_mbx_drv.get_mail();
  end

  for (genvar i = 0; i < NrMotors; i++) begin : g_motors
    vip_motor_sim #(
        .Idx(i)
    ) i_motor (
        .clk_i,
        .prescaler_i   (motor_prescaler),
        .enable_i      (motor_enable),
        .speed_target_i(),
        .speed_tune_i  (),
        .speed_real_o  (),
        .irq_o         (motor_irqs[i])
    );
  end

  vip_task_scoreboard i_scoreboard ();

  task automatic recv_letter(input logic [31:0] addr, input logic [31:0] data);
    unique case (addr)
      SimStartAddr: begin
        $display("Start trig");
        motor_enable = 1'b1;
      end
      SimEndAddr: begin
        $display("End trig");
        motor_enable = 1'b0;
      end
      default:
      $display("[VIP_SIM_ENV]: Warning! Received letter with unknown address: 0x%8h", addr);
    endcase
    /*
    $display("Dingdong: %h, %h", addr, data);
    if (addr == 32'h0400_0000) begin
      i_mbx_drv.send_letter(32'h1234_0000, 32'h0000_2222);
      i_mbx_drv.send_letter(32'h6767_6767, 32'h1234_1234);
      i_mbx_drv.raise_irq();
    end*/
  endtask

endmodule : vip_sim_env
