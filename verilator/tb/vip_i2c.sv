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

  assign scl_o = scl_i;

  bit          active_tx = 0;
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

  always @(posedge sda_i) begin : end_cond
    if (scl_i & active_tx) begin
      active_tx = 0;
    end
  end

  task handle_tx();
    automatic bit we;
    automatic logic [6:0] addr;
    automatic logic [7:0] data;

    receive_address(addr, we);
    $display("[I2C_VIP] Received addr %0d, we: %0d", addr, we);

    if (we) begin
      receive_data(data);
      $display("[I2C_VIP] Received data %0h", data);
    end else begin
      data = 'h5A;
      send_data(data);
      $display("[I2C_VIP] Sent data %0h", data);
    end

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
    g_counter = 0;
    sda_o = data[7];
    for (int unsigned i = 0; i < 7; i++) begin
      @(negedge scl_i);
      @(g_counter == InternalPrescaler / 2);
      sda_o = data[6-i];
    end
    @(negedge scl_i);
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
    // not strictly necessary, TODO for later
  endtask

  task receive_byte(output logic [7:0] data);
    for (int unsigned i = 0; i < 8; i++) begin
      @(posedge scl_i);
      @(g_counter == InternalPrescaler / 2);
      data[7-i] = sda_i;
    end
  endtask

endmodule : vip_i2c

