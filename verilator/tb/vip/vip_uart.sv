module vip_uart #(
) (
    input  logic clk_i,
    input  logic rst_ni,
    input  logic uart_rx_i,
    output logic uart_tx_o
);

  localparam int unsigned UartBaudRate = 115200;
  //bit uart_tip = 0;
  localparam int unsigned UartBaudPeriod = 1;  // 1000 / UartBaudRate;

  always @(negedge uart_rx_i) begin
    uart_read_byte();
  end

  task automatic uart_read_byte();
    automatic logic [7:0] bite = 0;
    #(UartBaudPeriod / 2);
    for (int i = 0; i < 8; i++) begin
      bite[i] = uart_rx_i;
    end
    /*
    $write("%c", bite);*/
    $display("Increment");

  endtask

endmodule : vip_uart

