// Copyright 2018 ETH Zurich and University of Bologna.
// Copyright and related rights are licensed under the Solderpad Hardware
// License, Version 0.51 (the "License"); you may not use this file except in
// compliance with the License.  You may obtain a copy of the License at
// http://solderpad.org/licenses/SHL-0.51. Unless required by applicable law
// or agreed to in writing, software, hardware and materials distributed under
// this License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
//
// Author: Florian Zaruba, ETH Zurich
// Date: 28/09/2018
// Description: Mock replacement for UART in testbench (not synthesiesable!)

module mock_uart (
    input  logic        clk_i,
    input  logic        rst_ni,
    input  logic        penable_i,
    input  logic        pwrite_i,
    input  logic [31:0] paddr_i,
    input  logic        psel_i,
    input  logic [31:0] pwdata_i,
    output logic [31:0] prdata_o,
    input  logic [ 3:0] pstrb_i,
    output logic        pready_o,
    output logic        pslverr_o
);
  localparam logic [7:0] RBR = 0;
  localparam logic [7:0] THR = 0;
  localparam logic [7:0] IER = 1;
  localparam logic [7:0] IIR = 2;
  localparam logic [7:0] FCR = 2;
  localparam logic [7:0] LCR = 3;
  localparam logic [7:0] MCR = 4;
  localparam logic [7:0] LSR = 5;
  localparam logic [7:0] MSR = 6;
  localparam logic [7:0] SCR = 7;
  localparam logic [7:0] DLL = 0;
  localparam logic [7:0] DLM = 1;

  localparam logic [7:0] THRE = 5;  // transmit holding register empty
  localparam logic [7:0] TEMT = 6;  // transmit holding register empty

  byte  lcr = 0;
  byte  dlm = 0;
  byte  dll = 0;
  byte  mcr = 0;
  byte  lsr = 0;
  byte  ier = 0;
  byte  msr = 0;
  byte  scr = 0;
  logic fifo_enabled = 1'b0;

  logic read_event, write_event;
  assign write_event = penable_i & psel_i & pwrite_i;
  assign read_event  = penable_i & psel_i & ~pwrite_i;
  byte wdata;
  /* verilator lint_off UNOPTFLAT*/
  logic [2:0] local_ofs;
  logic [2:0] local_addr;
  /* verilator lint_on UNOPTFLAT*/

  // APB assued to be word-alligned
  assign local_addr = {paddr_i[2], 2'b00} + local_ofs;

  always_comb begin

    wdata     = 0;
    local_ofs = 0;
    pslverr_o = 1'b0;

    if (psel_i & penable_i & pwrite_i) begin
      unique case (pstrb_i)
        4'b0001: begin
          local_ofs = 0;
          wdata    = pwdata_i[7:0];
        end
        4'b0010: begin
          local_ofs = 1;
          wdata    = pwdata_i[15:8];
        end
        4'b0100: begin
          local_ofs = 2;
          wdata    = pwdata_i[23:16];
        end
        4'b1000: begin
          local_ofs = 3;
          wdata    = pwdata_i[31:24];
        end
        default: begin

          pslverr_o = 1'b1;
          $warning("Warning: Illegal UART access");
        end
      endcase
    end
  end

  assign pready_o = 1'b1;

  function void uart_tx(byte ch);
    $write("%c", ch);
  endfunction : uart_tx

  /* verilator lint_off WIDTHTRUNC */
  /* verilator lint_off WIDTHEXPAND */
  /* verilator lint_off WIDTHCONCAT */

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (rst_ni) begin
      if (write_event) begin
        case (local_addr)
          THR: begin
            if (lcr & 'h80) dll <= wdata;
            else uart_tx(wdata);
          end
          IER: begin
            if (lcr & 'h80) dlm <= wdata;
            else ier <= wdata & 'hF;
          end
          FCR: begin
            if (wdata[0]) fifo_enabled <= 1'b1;
            else fifo_enabled <= 1'b0;
          end
          LCR: lcr <= wdata;
          MCR: mcr <= wdata & 'h1F;
          LSR: lsr <= wdata;
          MSR: msr <= wdata;
          SCR: scr <= wdata;
          default: ;
        endcase
      end
    end
  end

  function automatic logic [7:0] reg_read(logic [2:0] addr);
    logic [7:0] r;
    r = 8'h0;
    case (addr)
      RBR: if (lcr & 'h80) r = dll;
      IER: begin
        if (lcr & 'h80) r = dlm;
        else r = ier;
      end
      IIR: r = fifo_enabled ? 8'hc0 : 8'h00;
      LCR: r = lcr;
      MCR: r = mcr;
      LSR: r = lsr | (1 << THRE) | (1 << TEMT);
      MSR: r = msr;
      SCR: r = scr;
      default: ;
    endcase
    return r;
  endfunction : reg_read

  always_comb begin
    prdata_o = '0;
    if (read_event) begin
      for (int unsigned i = 0; i < 4; i++) begin
        prdata_o[8*i+:8] = reg_read({paddr_i[2], 2'b00} + 3'(i));
      end
    end
  end

  /* verilator lint_on WIDTHTRUNC */
  /* verilator lint_on WIDTHEXPAND */
  /* verilator lint_on WIDTHCONCAT */

endmodule

