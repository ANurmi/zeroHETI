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
    input  logic          clk_i,
    input  logic          rst_ni,
    input  logic          penable_i,
    input  logic          pwrite_i,
    input  logic [31:0]   paddr_i,
    input  logic          psel_i,
    input  logic [31:0]   pwdata_i,
    output logic [31:0]   prdata_o,
    input  logic  [3:0]   pstrb_i,
    output logic          pready_o,
    output logic          pslverr_o
);
    localparam RBR = 0;
    localparam THR = 0;
    localparam IER = 1;
    localparam IIR = 2;
    localparam FCR = 2;
    localparam LCR = 3;
    localparam MCR = 4;
    localparam LSR = 5;
    localparam MSR = 6;
    localparam SCR = 7;
    localparam DLL = 0;
    localparam DLM = 1;

    localparam THRE = 5; // transmit holding register empty
    localparam TEMT = 6; // transmit holding register empty

    byte lcr = 0;
    byte dlm = 0;
    byte dll = 0;
    byte mcr = 0;
    byte lsr = 0;
    byte ier = 0;
    byte msr = 0;
    byte scr = 0;
    logic fifo_enabled = 1'b0;

    bit write_event = penable_i & psel_i &  pwrite_i;
    bit read_event  = penable_i & psel_i & ~pwrite_i;
    byte wdata, rdata;
    logic [2:0] local_ofs;
/* verilator lint_off UNOPTFLAT*/
    logic [2:0] local_addr;
/* verilator lint_on UNOPTFLAT*/

    assign local_addr = paddr_i[2:0] + local_ofs;

    always_comb begin
      prdata_o  = 0;
      wdata     = 0;
      local_ofs = 0;

      unique case (pstrb_i)
        4'b0001: begin
          local_ofs = 0;
          prdata_o = {24'h0, rdata};
          wdata    = pwdata_i[7:0];
        end
        4'b0010: begin
          local_ofs = 1;
          prdata_o = {16'h0, rdata, 8'h0};
          wdata    = pwdata_i[15:8];
        end
        4'b0100: begin
          local_ofs = 2;
          prdata_o = {8'h0, rdata, 16'h0};
          wdata    = pwdata_i[23:16];
        end
        4'b1000: begin
          local_ofs = 3;
          prdata_o = {rdata, 24'h0};
          wdata    = pwdata_i[31:24];
        end
        default: begin
          if (psel_i & penable_i) $fatal(1, "Fatal: Illegal UART access");
        end
      endcase
    end


    assign pready_o = 1'b1;
    assign pslverr_o = 1'b0;

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
                    default:;
                endcase
            end
        end
    end

    always_comb begin
        if (read_event) begin
            case (local_addr)
                THR: begin
                    if (lcr & 'h80) rdata = {24'b0, dll};
                end
                IER: begin
                    if (lcr & 'h80) rdata = {24'b0, dlm};
                    else rdata = {24'b0, ier};
                end
                IIR: begin
                    if (fifo_enabled) rdata = {24'b0, 8'hc0};
                    else rdata = {24'b0, 8'b0};
                end
                LCR: rdata = {24'b0, lcr};
                MCR: rdata = {24'b0, mcr};
                LSR: rdata = {24'b0, (lsr | (1 << THRE) | (1 << TEMT))};
                MSR: rdata = {24'b0, msr};
                SCR: rdata = {24'b0, scr};
                default:;
            endcase
        end
    end

/* verilator lint_on WIDTHTRUNC */
/* verilator lint_on WIDTHEXPAND */
/* verilator lint_on WIDTHCONCAT */

endmodule

