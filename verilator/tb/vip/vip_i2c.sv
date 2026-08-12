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

  rt_prof_pkg::i2c_transaction_t tx_state = '{default: 0};

  assign scl_o = scl_i;

  initial begin
    sda_o = 1'b1;
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
        //vip_req_o.write       = tx_state.data[0];
        //vip_req_o.addr        = tx_state.data[7:1];
        @(posedge clk_i);
        if (!tx_state.is_write) begin
          sda_o = 0;//vip_rsp_i.rdata[7];
        end
        tx_state.data = 0;
      end

    end else begin
      // Data
      //vip_req_o.valid = 0;
      if (tx_state.is_write) begin : write
        if (tx_state.bitcount < 9) begin
          tx_state.data[8-tx_state.bitcount] = sda_i;
          if (tx_state.bitcount == 8) sda_o = 0;
        end else begin
          //vip_req_o.valid = 1;
          //vip_req_o.wdata = tx_state.data;
          tx_state.bitcount = 0;
          sda_o = 1'b1;
        end
      end : write

      else begin : read
        if (tx_state.bitcount < 9) begin
          sda_o = 0;  //vip_rsp_i.rdata[7-tx_state.bitcount];
        end else begin
          //vip_req_o.valid = 1;
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
      tx_state = '{default: 0};
      //vip_req_o.write = 0;
      //vip_req_o.valid = 0;
      sda_o    = 1'b1;
    end
  end : stop_condition

endmodule : vip_i2c

