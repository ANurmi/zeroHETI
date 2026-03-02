// Copyright 2023 ETH Zurich and University of Bologna.
// Solderpad Hardware License, Version 0.51, see LICENSE for details.
// SPDX-License-Identifier: SHL-0.51

// Michael Rogenmoser <michaero@iis.ee.ethz.ch>

`include "obi/typedef.svh"
`include "obi/assign.svh"

module obi_cut_intf #(
  parameter obi_pkg::obi_cfg_t ObiCfg  = obi_pkg::ObiDefaultConfig,
  bit Bypass = 1'b0
)(
  input logic         clk_i,
  input logic         rst_ni,
  OBI_BUS.Manager     obi_m,
  OBI_BUS.Subordinate obi_s
);

typedef logic [31:0] addr_t;

  spill_register #(
    .T      ( addr_t),
    .Bypass ( Bypass   )
  ) i_reg_a (
    .clk_i,
    .rst_ni,
    .valid_i ( obi_s.req ),
    .ready_o ( obi_s.gnt ),
    .data_i  ( obi_s.addr ),
    .valid_o ( obi_m.req ),
    .ready_i ( obi_m.gnt ),
    .data_o  ( obi_m.addr )
  );

  spill_register #(
    .T      ( addr_t),
    .Bypass ( Bypass    )
  ) i_req_r (
    .clk_i,
    .rst_ni,
    .valid_i ( obi_m.rvalid ),
    .ready_o ( ),
    .data_i  ( obi_m.rdata      ),
    .valid_o ( obi_s.rvalid ),
    .ready_i ( 1'b1 ),
    .data_o  ( obi_s.rdata      )
  );

endmodule
