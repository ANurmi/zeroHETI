module apb_i2c #(
    parameter int unsigned APB_ADDR_WIDTH = 12  //APB slaves are 4KB by default
) (
    input  logic                      HCLK,
    input  logic                      HRESETn,
    input  logic [APB_ADDR_WIDTH-1:0] PADDR,
    input  logic [              31:0] PWDATA,
    input  logic                      PWRITE,
    input  logic                      PSEL,
    input  logic                      PENABLE,
    output logic [              31:0] PRDATA,
    output logic                      PREADY,
    output logic                      PSLVERR,
    output logic                      interrupt_o,
    input  logic                      scl_pad_i,
    output logic                      scl_pad_o,
    output logic                      scl_padoen_o,
    input  logic                      sda_pad_i,
    output logic                      sda_pad_o,
    output logic                      sda_padoen_o
);

  localparam logic [2:0] RegClkPrescaler = 3'b000;  //BASEADDR+0x00
  localparam logic [2:0] RegCtrl = 3'b001;  // BASEADDR+0x04
  localparam logic [2:0] RegRx = 3'b010;  // BASEADDR+0x08
  localparam logic [2:0] RegStatus = 3'b011;  // BASEADDR+0x0C
  localparam logic [2:0] RegTx = 3'b100;  // BASEADDR+0x10
  localparam logic [2:0] RegCmd = 3'b101;  // BASEADDR+0x14

  //
  // variable declarations
  //

  logic [ 2:0] s_apb_addr;

  // registers
  reg   [15:0] r_pre;  // clock prescale register
  reg   [ 7:0] r_ctrl;  // control register
  reg   [ 7:0] r_tx;  // transmit register
  wire  [ 7:0] s_rx;  // receive register
  reg   [ 7:0] r_cmd;  // command register
  wire  [ 7:0] s_status;  // status register

  // done signal: command completed, clear command register
  wire         s_done;

  // core enable signal
  wire         s_core_en;
  wire         s_ien;

  // status register signals
  wire         s_irxack;
  reg          rxack;  // received aknowledge from slave
  reg          tip;  // transfer in progress
  reg          irq_flag;  // interrupt pending flag
  wire         i2c_busy;  // bus busy (start signal detected)
  wire         i2c_al;  // i2c bus arbitration lost
  reg          al;  // status register arbitration lost bit

  //
  // module body
  //

  assign s_apb_addr = PADDR[4:2];

  always_ff @(posedge HCLK, negedge HRESETn) begin
    if (~HRESETn) begin
      r_pre  <= 'h0;
      r_ctrl <= 'h0;
      r_tx   <= 'h0;
      r_cmd  <= 'h0;
    end else if (PSEL && PENABLE && PWRITE) begin
      if (s_done | i2c_al) r_cmd[7:4] <= 4'h0;  // clear command bits when done
                                                // or when aribitration lost
      r_cmd[2:1] <= 2'b0;  // reserved bits
      r_cmd[0]   <= 1'b0;  // clear IRQ_ACK bit
      case (s_apb_addr)
        RegClkPrescaler: r_pre <= PWDATA[15:0];
        RegCtrl: r_ctrl <= PWDATA[7:0];
        RegTx: r_tx <= PWDATA[7:0];
        RegCmd: begin
          if (s_core_en) r_cmd <= PWDATA[7:0];
        end
        default: ;
      endcase
    end else begin
      if (s_done | i2c_al) r_cmd[7:4] <= 4'h0;  // clear command bits when done
                                                // or when aribitration lost
      r_cmd[2:1] <= 2'b0;  // reserved bits
      r_cmd[0]   <= 1'b0;  // clear IRQ_ACK bit
    end
  end  //always

  always_comb begin
    case (s_apb_addr)
      RegClkPrescaler: PRDATA = {16'h0, r_pre};
      RegCtrl: PRDATA = {24'h0, r_ctrl};
      RegRx: PRDATA = {24'h0, s_rx};
      RegStatus: PRDATA = {24'h0, s_status};
      RegTx: PRDATA = {24'h0, r_tx};
      RegCmd: PRDATA = {24'h0, r_cmd};
      default: PRDATA = 'h0;
    endcase
  end

  // decode command register
  wire sta = r_cmd[7];
  wire sto = r_cmd[6];
  wire rd = r_cmd[5];
  wire wr = r_cmd[4];
  wire ack = r_cmd[3];
  wire iack = r_cmd[0];

  // decode control register
  assign s_core_en = r_ctrl[7];
  assign s_ien     = r_ctrl[6];

  // hookup byte controller block
  i2c_master_byte_ctrl byte_controller (
      .clk     (HCLK),
      .nReset  (HRESETn),
      .ena     (s_core_en),
      .clk_cnt (r_pre),
      .start   (sta),
      .stop    (sto),
      .read    (rd),
      .write   (wr),
      .ack_in  (ack),
      .din     (r_tx),
      .cmd_ack (s_done),
      .ack_out (s_irxack),
      .dout    (s_rx),
      .i2c_busy(i2c_busy),
      .i2c_al  (i2c_al),
      .scl_i   (scl_pad_i),
      .scl_o   (scl_pad_o),
      .scl_oen (scl_padoen_o),
      .sda_i   (sda_pad_i),
      .sda_o   (sda_pad_o),
      .sda_oen (sda_padoen_o)
  );

  // status register block + interrupt request signal
  always_ff @(posedge HCLK, negedge HRESETn) begin
    if (!HRESETn) begin
      al       <= 1'b0;
      rxack    <= 1'b0;
      tip      <= 1'b0;
      irq_flag <= 1'b0;
    end else begin
      al <= i2c_al | (al & ~sta);
      rxack <= s_irxack;
      tip <= (rd | wr);
      // interrupt request flag is always generated
      irq_flag <= (s_done | i2c_al | irq_flag) & ~iack;
    end
  end

  // generate interrupt request signals
  always_ff @(posedge HCLK, negedge HRESETn) begin
    if (!HRESETn) interrupt_o <= 1'b0;
    else
      // interrupt signal is only generated when IEN (interrupt enable bit is set)
      interrupt_o <= irq_flag && s_ien;
  end

  // assign status register bits
  assign s_status[7] = rxack;
  assign s_status[6] = i2c_busy;
  assign s_status[5] = al;
  assign s_status[4:2] = 3'h0;  // reserved
  assign s_status[1] = tip;
  assign s_status[0] = irq_flag;

  assign PREADY = 1'b1;
  assign PSLVERR = 1'b0;

endmodule
