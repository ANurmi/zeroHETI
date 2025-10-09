module irq_gateway #(
  parameter int unsigned NrInputs = 32
)(
  input  logic                clk_i,
  input  logic                rst_ni,
  input  logic [NrInputs-1:0] trig_type_i,
  input  logic [NrInputs-1:0] trig_polarity_i,
  input  logic [NrInputs-1:0] irqs_i,
  output logic [NrInputs-1:0] irqs_o
);
endmodule : irq_gateway

