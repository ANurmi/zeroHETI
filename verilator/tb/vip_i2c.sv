module vip_i2c #(
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic sda_i,
    output logic sda_o,
    input  logic scl_i,
    output logic scl_o
);

  localparam InternalPrescaler = 16;
  localparam BufLen = 4;

  assign scl_o = scl_i;

  bit          active_tx = 0;
  bit          active_byte = 0;
  int unsigned g_counter = 0;

  initial begin
    sda_o = 1'b1;
  end

  always @(posedge clk_i) begin : counters
    // # delays are a no-no with Verilator
    if (active_tx) begin
      if (g_counter == InternalPrescaler - 1) begin
        g_counter = 0;
      end else g_counter++;
    end
  end

  always @(negedge scl_i) begin : start_cond
    if (!sda_i & !active_tx) begin
      active_tx = 1;
      handle_tx();
    end
  end

  vip_ctrl_sim i_ctrl_sim (
    .clk_i
  );

  task handle_tx();
    automatic bit we;
    automatic logic [6:0] addr;
    automatic logic [7:0] data;

    automatic logic [31:0] write_buf = 0;
    automatic logic [31:0] read_buf  = 0;

    automatic int unsigned idx = 0;
    
    active_byte = 1;
    receive_address(addr, we);
    active_byte = 0;

    delay_half(4);

    if (!we) i_ctrl_sim.read(addr, read_buf);

    while (!scl_i) begin
      active_byte = 1;

      if (we) begin
        receive_data(data);
        write_buf[(idx*8) +: 8] = data;
      end else begin
        data = read_buf[(idx*8) +: 8];
        send_data(data);
      end

      idx = (idx == BufLen-1) ? 0 : idx + 1;

      active_byte = 0;
      delay_half(4);
      
    end

    // Model writeback
    if (we) i_ctrl_sim.write(addr, write_buf);

    active_tx = 0;

  endtask

  task receive_address(output logic [6:0] addr, output bit we);
    automatic logic [7:0] rbyte;

    receive_byte(rbyte);
    addr = rbyte[7:1];
    we   = rbyte[0];
    send_ack();
  endtask

  task receive_data(output logic [7:0] data);
    receive_byte(data);
    send_ack();
  endtask

  task send_data(input logic [7:0] data);
    sda_o = data[7];
    for (int unsigned i = 0; i < 8; i++) begin
      @(negedge scl_i);
      delay_half(1);
      sda_o = data[6-i];
    end
    sda_o = 1'b1;
    receive_ack();
  endtask

  task send_ack();
    @(negedge scl_i);
    @(g_counter == InternalPrescaler / 2);
    sda_o = 0;
    @(negedge scl_i);
    @(g_counter == InternalPrescaler / 2);
    sda_o = 1;
  endtask

  task receive_ack();
    @(negedge sda_i);
    @(negedge scl_i);
  endtask

  task receive_byte(output logic [7:0] data);
    g_counter = 0;
    for (int unsigned i = 0; i < 8; i++) begin
      @(posedge scl_i);
      @(g_counter == InternalPrescaler / 2);
      data[7-i] = sda_i;
    end
  endtask

  task delay_half(input int count);
    for (int i = 0; i < count; i++) begin
      g_counter = 0;
      @(g_counter == InternalPrescaler / 2);
    end
  endtask

endmodule : vip_i2c

