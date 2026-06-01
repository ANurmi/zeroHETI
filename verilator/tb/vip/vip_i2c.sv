module vip_i2c #(
    parameter type req_t = logic,
    parameter type rsp_t = logic
) (
    input  logic       clk_i,
    input  logic       rst_ni,
    input  logic       sda_i,
    output logic       sda_o,
    input  logic       scl_i,
    output logic       scl_o,
    output logic [3:0] irq_o,
    output req_t       vip_req_o,
    input  rsp_t       vip_rsp_i
);
  /*
  localparam int unsigned InternalPrescaler = 16;
  localparam int unsigned BufLen = 4;
*/
  typedef struct packed {
    bit          active;
    bit          frame_active;
    bit          addr_valid;
    bit          is_write;
    logic [7:0]  data;
    int unsigned bitcount;
  } transaction_t;

  transaction_t tx_state = '{default: 0};
  /*
  bit                transaction = 0;
  bit                frame = 0;
  bit                addr_valid = 0;
  bit                stop_cond = 0;
  //int unsigned       g_counter = 0;
  int unsigned       bitcount = 0;

  logic        [7:0] rx_byte = 0;
*/
  assign scl_o = scl_i;

  initial begin
    sda_o = 1'b1;
    //transaction_t tx_state = '{default: 0};
  end

  always @(posedge scl_i) begin : bit_counter
    if (tx_state.active) tx_state.bitcount++;
    else tx_state.bitcount = 0;
  end : bit_counter

  // verilator lint_off LATCH
  always @(tx_state.bitcount) begin

    tx_state.frame_active = 1;

    @(negedge scl_i);

    // Address
    if (!tx_state.addr_valid) begin

      if (tx_state.bitcount < 9) begin
        tx_state.data[8-tx_state.bitcount] = sda_i;
        if (tx_state.bitcount == 8) sda_o = 0;
      end else begin
        sda_o                 = 1;
        tx_state.addr_valid   = 1;
        tx_state.bitcount     = 0;
        tx_state.frame_active = 0;
        tx_state.is_write     = tx_state.data[0];
        vip_req_o.write       = tx_state.data[0];
        vip_req_o.addr        = tx_state.data[7:1];
        if (!tx_state.is_write) begin
          sda_o = vip_rsp_i.rdata[7];
        end
        tx_state.data = 0;
      end

    end else begin
      // Data
      vip_req_o.valid = 0;
      if (tx_state.is_write) begin : write
        if (tx_state.bitcount < 9) begin
          tx_state.data[8-tx_state.bitcount] = sda_i;
          if (tx_state.bitcount == 8) sda_o = 0;
        end else begin
          vip_req_o.valid = 1;
          vip_req_o.wdata = tx_state.data;
          tx_state.bitcount = 0;
          sda_o = 1'b1;
        end
      end : write

      else begin : read
        if (tx_state.bitcount < 9) begin
          sda_o = vip_rsp_i.rdata[7-tx_state.bitcount];
        end else begin
          vip_req_o.valid = 1;
          sda_o = 1'b1;
          tx_state.bitcount = 0;
        end
      end : read
    end

  end
  // verilator lint_on LATCH

  always @(negedge scl_i) begin : start_condition
    if (!sda_i) begin
      tx_state.active       = 1;
      tx_state.frame_active = 1;
    end
  end : start_condition

  always @(posedge sda_i) begin : stop_condition
    if (tx_state.active & scl_i) begin
      tx_state        = '{default: 0};
      vip_req_o.write = 0;
      vip_req_o.valid = 0;
      sda_o           = 1'b1;
    end
  end : stop_condition


  /*
  always @(posedge clk_i) begin : counters
    if (active_tx) begin
      if (g_counter == InternalPrescaler - 1) begin
        g_counter = 0;
      end else g_counter++;
    end
  end

  always @(negedge scl_i) begin : g_start_cond
    if (!sda_i & !active_tx) begin
      vip_req_o.valid = 0;
      stop_cond = 0;
      active_tx = 1;
      handle_tx();
    end
  end

  always @(posedge sda_i) begin : g_stop_cond
    if (active_tx & scl_i) stop_cond = 1;
  end

  task automatic handle_tx();

    automatic logic [7:0] rbyte;

    receive_byte(rbyte);

    vip_req_o.addr  = rbyte[7:1];
    vip_req_o.write = rbyte[0];

    if (vip_req_o.write) begin

      automatic logic [31:0] wbuf;

      for (int i = 0; !stop_cond; i++) begin
        automatic logic [7:0] wbyte;
        receive_byte(wbyte);
        wbuf[(8*i)+:8] = wbyte;
      end

      vip_req_o.valid = 1;
      vip_req_o.wdata = wbuf;

    end else begin

      vip_req_o.valid = 1;
      delay_half(2);

      for (int i = 0; !stop_cond; i++) begin
        delay_half(2);
        send_byte(vip_rsp_i.rdata[(8*i)+:8]);
      end

    end

    active_tx = 0;

  endtask

  task automatic receive_byte(output logic [7:0] rbyte);

    active_byte = 1;

    for (int i = 0; i < 8; i++) begin
      @(posedge scl_i);
      rbyte[7-i] = sda_i;
    end

    delay_half(1);
    rbyte[0] = sda_i;
    delay_half(6);
    send_ack();
    active_byte = 0;
    delay_half(8);

  endtask

  task automatic send_byte(input logic [7:0] dbyte);

    active_byte = 1;
    sda_o = dbyte[7];

    for (int i = 0; i < 9; i++) begin
      @(negedge scl_i);
      delay_half(2);
      if (i == 8) begin
      end else begin
        sda_o = dbyte[6-i];
      end
    end

    sda_o = 1;
    active_byte = 0;
    delay_half(8);

  endtask

  task automatic send_ack();
    sda_o = 0;
    @(negedge scl_i);
    delay_half(1);
    sda_o = 1;
  endtask

  task automatic delay_half(input int count);
    for (int i = 0; i < count; i++) begin
      g_counter = 0;
      @(g_counter == InternalPrescaler / 2);
    end
  endtask
*/
endmodule : vip_i2c

