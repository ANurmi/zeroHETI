module irq_gateway #(
  parameter int unsigned NrInputs = 32
)(
  input  logic                clk_i,
  input  logic                rst_ni,
  input  logic [NrInputs-1:0] trig_type_i,
  input  logic [NrInputs-1:0] trig_polarity_i,
  input  logic [NrInputs-1:0] irqs_i,
  input  logic [NrInputs-1:0] ip_i,
  output logic [NrInputs-1:0] irqs_o
);

logic [NrInputs-1:0] irqs_d, irqs_q;

assign irqs_d = irqs_i;

always @(posedge clk_i) begin
  if (~rst_ni) begin
    irqs_q <= '0;
  end else begin
    irqs_q <= irqs_d;
  end
end

for (genvar i=0; i<NrInputs; i++) begin : g_lines

  always_comb begin : output_assign

    irqs_o[i] = 1'b0;

    unique case ({trig_polarity_i[i], trig_type_i[i]})
      2'b00: begin // positive edge-triggered
        irqs_o[i] = irqs_i[i] & ~irqs_q[i];
      end
      2'b01: begin // positive level-triggered
        irqs_o[i] = irqs_i[i] & irqs_q[i] & ~ip_i[i];
      end
      2'b10: begin // negative edge-triggered
        irqs_o[i] = ~irqs_i[i] & irqs_q[i];
      end
      2'b11: begin // negative level-triggered
        irqs_o[i] = ~irqs_i[i] & ~irqs_q[i] & ~ip_i[i];
      end
      default:;
    endcase

  end : output_assign

end : g_lines

endmodule : irq_gateway

