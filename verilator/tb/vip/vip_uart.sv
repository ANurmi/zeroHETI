// Simple UART receiver
// Baudrate hardcoded to 1_500_000;
// TODO: parameterize

module vip_uart #(
) (
    input  logic clk_i,
    input  logic rx_i,
    output logic tx_o
);

  localparam int unsigned PsMax = 80;

  logic        [7:0] char;

  int unsigned       bit_counter = 0;
  int unsigned       prescaler = 0;
  bit                enable = 0;

  always @(negedge rx_i) begin
    if (~enable) begin
      enable = 1;
    end
  end

  always @(posedge rx_i) begin
    if (bit_counter >= 8) begin
      enable = 0;
      bit_counter = 0;
    end
  end

  always @(posedge clk_i) begin
    if (enable) begin
      if (prescaler >= PsMax - 1) begin
        char[bit_counter-1] = rx_i;
        bit_counter++;
        prescaler = 0;
      end else begin
        prescaler++;
      end
    end
  end

  always @(negedge enable) begin
    $write("%c", char);
    char = 0;
  end

endmodule : vip_uart
