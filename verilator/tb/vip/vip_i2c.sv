module vip_i2c #(
) (
    input  logic       clk_i,
    input  logic       rst_ni,
    input  logic       sda_i,
    output logic       sda_o,
    input  logic       scl_i,
    output logic       scl_o,
    output logic [3:0] irq_o
);
  typedef struct packed {
    bit          active;
    bit          frame_active;
    bit          byte_active;
    bit          addr_valid;
    bit          write;
    int unsigned bitcount;
    logic [7:0]  addr;
    logic [7:0]  rdata;
    logic [7:0]  wdata;
  } i2c_transaction_t;

  i2c_transaction_t tx_state = '{default: 0};

  initial begin
    sda_o = 1'b1;
  end

  assign scl_o = scl_i;

  always @(posedge scl_i) begin : bit_counter
    if (tx_state.active) tx_state.bitcount++;
    else tx_state.bitcount = 0;
  end : bit_counter

  always @(tx_state.bitcount) begin

    tx_state.frame_active = 1'b1;
    @(negedge scl_i);

    if (tx_state.bitcount < 9) begin

      tx_state.byte_active = 1'b1;

      if (!tx_state.addr_valid) begin : addr
        tx_state.addr[8-tx_state.bitcount] = sda_i;
        if (tx_state.bitcount == 8) sda_o = 1'b0;
      end : addr

      else if (tx_state.write) begin : write
        tx_state.wdata[8-tx_state.bitcount] = sda_i;
        if (tx_state.bitcount == 8) sda_o = 1'b0;
      end : write

      else begin : read
        sda_o = tx_state.rdata[7-tx_state.bitcount];
      end : read

    end else begin
      sda_o                 = 1'b1;
      tx_state.byte_active  = 1'b0;
      tx_state.addr_valid   = 1'b1;
      tx_state.bitcount     = '0;
      tx_state.frame_active = 1'b0;
      tx_state.write        = tx_state.addr[0];
      if (tx_state.write) begin
        // TODO: scoreboard hook here
      end
      @(posedge clk_i);
      if (!tx_state.write) begin
        sda_o = tx_state.rdata[7];
      end
    end
  end

  always @(negedge scl_i) begin : start_condition
    if (!sda_i) begin
      tx_state.active       = 1;
      tx_state.frame_active = 1;
    end
  end : start_condition

  always @(posedge sda_i) begin : stop_condition
    if (tx_state.active & scl_i) begin
      tx_state.active       = 0;
      tx_state.frame_active = 0;
      tx_state.byte_active  = 0;
      tx_state.addr_valid   = 0;
      tx_state.write        = 0;
      tx_state.bitcount     = 0;
      sda_o                 = 1'b1;
    end
  end : stop_condition

  function automatic logic [6:0] get_addr();
    return tx_state.addr[7:1];
  endfunction

  function automatic void set_rdata_byte(input logic [7:0] data);
    tx_state.rdata = data;
  endfunction

  function automatic logic [7:0] get_wdata_byte();
    return tx_state.wdata;
  endfunction

endmodule : vip_i2c

