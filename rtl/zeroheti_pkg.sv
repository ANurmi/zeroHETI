package zeroheti_pkg;

  localparam logic [31:0] BootAddr = 32'h0800;

  typedef enum integer {
    HETIC = 0,
    CLIC  = 1,
    EDFIC = 2
  } int_ctrl_e;

  localparam int_ctrl_e IntController = `INTC;

  typedef struct packed {
    bit rve;
    bit bt_alu;
    bit wb_stage;
    ibex_pkg::rv32m_e mul;
    int_ctrl_e ic;
    int unsigned size_tg;
    int unsigned num_irqs;
    int unsigned num_prio;
    int unsigned hart_id;
    logic [31:0] boot_addr;
  } core_cfg_t;


  localparam core_cfg_t MinCfg = '{
      rve       : 1,
      bt_alu    : 0,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MNone,
      ic        : IntController,
      size_tg   : 4,
      num_irqs  : 16,
      num_prio  : 8,
      hart_id   : 0,
      boot_addr : BootAddr
  };

  localparam core_cfg_t DefaultCfg = '{
      rve       : 1,
      bt_alu    : 1,
      wb_stage  : 1,
      mul       : ibex_pkg::RV32MSingleCycle,
      ic        : IntController,
      size_tg   : 16,
      num_irqs  : 128,
      num_prio  : 128,
      hart_id   : 0,
      boot_addr : BootAddr
  };

  typedef struct packed {
    logic [31:0] base;
    logic [31:0] last;
  } addr_rule_t;

  typedef struct packed {
    addr_rule_t dbg;
    addr_rule_t imem;
    addr_rule_t dmem;
    addr_rule_t intc;
    addr_rule_t uart;
    addr_rule_t i2c_0;
    addr_rule_t i2c_1;
    addr_rule_t spi;
    addr_rule_t tg;
    addr_rule_t cfg;
    addr_rule_t mtimer;
    addr_rule_t mbx;
    addr_rule_t ext;
  } addr_map_t;

  localparam int unsigned ImemSize = `IMEM_BYTES;
  localparam int unsigned DmemSize = `DMEM_BYTES;

  localparam addr_rule_t DbgAddr = '{base : 32'h0000_0000, last : 32'h0000_1000};
  localparam addr_rule_t UartAddr = '{base : 32'h0000_3000, last : 32'h0000_3100};
  localparam addr_rule_t MtimerAddr = '{base : 32'h0000_3100, last : 32'h0000_3114};
  localparam addr_rule_t I2c0Addr = '{base : 32'h0000_3200, last : 32'h0000_3300};
  localparam addr_rule_t I2c1Addr = '{base : 32'h0000_3300, last : 32'h0000_3400};
  localparam addr_rule_t TimerGroupAddr = '{
      base : 32'h0000_3400,
      last : 32'h0000_3400 + (16 * DefaultCfg.size_tg)
  };
  localparam addr_rule_t CfgAddr = '{base : 32'h0000_4000, last : 32'h0000_5000};
  localparam addr_rule_t SpiAddr = '{base : 32'h0000_5000, last : 32'h0000_5100};
  localparam addr_rule_t ImemAddr = '{base : 32'h0001_0000, last : (32'h0001_0000 + ImemSize)};
  localparam addr_rule_t DmemAddr = '{base : 32'h0002_0000, last : (32'h0002_0000 + DmemSize)};
  localparam addr_rule_t IntcAddr = '{base : 32'h0010_0000, last : 32'h0010_2000};
  localparam addr_rule_t MbxAddr = '{base : 32'h0010_8000, last : 32'h0010_8100};
  localparam addr_rule_t ExtAddr = '{base : 32'h0011_0000, last : 32'hFFFF_FFFF};

  // imem size in words
  localparam int unsigned ImemWSize = ImemSize / 4;
  // dmem size in words
  localparam int unsigned DmemWSize = DmemSize / 4;

  localparam addr_map_t AddrMap = '{
      dbg    : DbgAddr,
      imem   : ImemAddr,
      dmem   : DmemAddr,
      intc   : IntcAddr,
      uart   : UartAddr,
      i2c_0  : I2c0Addr,
      i2c_1  : I2c1Addr,
      cfg    : CfgAddr,
      spi    : SpiAddr,
      tg     : TimerGroupAddr,
      mtimer : MtimerAddr,
      mbx    : MbxAddr,
      ext    : ExtAddr
  };

  // Locally used OBI typedefs
  typedef struct packed {
    logic [31:0] addr;
    logic        we;
    logic [3:0]  be;
    logic [31:0] wdata;
    logic        aid;
    logic        a_optional;
  } obi_a_chan_t;

  typedef struct packed {
    logic [31:0] rdata;
    logic rid;
    logic err;
    logic r_optional;
  } obi_r_chan_t;

  typedef struct packed {
    obi_a_chan_t a;
    logic req;
  } obi_req_t;

  typedef struct packed {
    obi_r_chan_t r;
    logic gnt;
    logic rvalid;
  } obi_rsp_t;


endpackage
