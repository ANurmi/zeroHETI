module irq_arbiter #(
  parameter  int unsigned NrInputs  = 32,
  parameter  int unsigned PrioWidth = 8,
  localparam int unsigned IdxWidth  = $clog2(NrInputs)
)(
  input  logic [NrInputs-1:0]                valid_i,
  output logic                               valid_o,
  input  logic [NrInputs-1:0][PrioWidth-1:0] prio_i,
  output logic               [PrioWidth-1:0] prio_o,
  output logic                [IdxWidth-1:0] idx_o
);

localparam int unsigned NrNodes = NrInputs-1;

typedef struct packed {
  logic [PrioWidth-1:0] prio;
  logic  [IdxWidth-1:0] idx;
  logic                 valid;
} node_t;

typedef enum logic {SEL_A, SEL_B} sel_e;

// 5.008 does not like multiple assignments to the
// "same" signal, hence this.

// verilator lint_off UNOPTFLAT
node_t [NrNodes-1:0] nodes;
// verilator lint_on UNOPTFLAT

// Extract outputs from top sorting node
assign valid_o = nodes[NrNodes-1].valid;
assign idx_o   = nodes[NrNodes-1].idx;
assign prio_o  = nodes[NrNodes-1].prio;

for (genvar i=0; i<IdxWidth; i++) begin : g_levels

 if (i == IdxWidth-1) begin : g_input
    for (genvar k=0; k<NrInputs/2; k++) begin : g_result

      localparam int unsigned IdxA = 2*k;
      localparam int unsigned IdxB = 2*k+1;

      sel_e sel;
      assign sel = arbitrate( valid_i[IdxA], valid_i[IdxB],
                    prio_i[IdxA], prio_i[IdxB]);

      assign nodes[k].prio  = (sel == SEL_A) ? prio_i[IdxA]    : prio_i[IdxB];
      assign nodes[k].valid = (sel == SEL_A) ? valid_i[IdxA]   : valid_i[IdxB];
      assign nodes[k].idx   = (sel == SEL_A) ? IdxWidth'(IdxA) : IdxWidth'(IdxB);

    end : g_result
  end : g_input

  else begin : g_sort_nodes

    localparam int unsigned LayerBaseIdx  = NrInputs-(2**(i+2));
    localparam int unsigned OutputBaseIdx = NrInputs-(2**(i+1));
    localparam int unsigned LayerSizeOut  = 2**i;

    for (genvar k=0; k<LayerSizeOut; k++) begin : g_result

      localparam int unsigned IdxA = LayerBaseIdx  + 2*k;
      localparam int unsigned IdxB = LayerBaseIdx  + 2*k+1;
      localparam int unsigned IdxO = OutputBaseIdx + k;

      sel_e sel;
      assign sel = arbitrate( nodes[IdxA].valid, nodes[IdxB].valid,
                    nodes[IdxA].prio, nodes[IdxB].prio);

      assign nodes[IdxO] = (sel == SEL_A) ? nodes[IdxA] : nodes[IdxB];

    end : g_result

  end : g_sort_nodes

end : g_levels

function sel_e arbitrate(
  logic valid_a, 
  logic valid_b, 
  logic [PrioWidth-1:0] prio_a, 
  logic [PrioWidth-1:0] prio_b, 
);
  sel_e sel;
  sel = SEL_A;

  // B must be valid to affect default
  if(valid_b) begin
    // B wins based on validity alone
    if (~valid_a) sel = SEL_B;
    // both inputs valid => comparison required
    else if (prio_a < prio_b) sel = SEL_B;
  end

  return sel;

endfunction

endmodule : irq_arbiter

