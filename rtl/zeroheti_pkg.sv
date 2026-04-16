package zeroheti_pkg;

  localparam logic [31:0] BootAddr = 32'h0800;

  typedef enum integer {
    //HETIC = 0, Deprecated
    CLIC  = 1,
    EDFIC = 2
  } int_ctrl_e;

  localparam int_ctrl_e IntController = `INTC;

  typedef struct packed {
    int_ctrl_e   ic;
    int unsigned num_irqs;
    int unsigned num_prio;
    int unsigned hart_id;
    logic [31:0] boot_addr;
  } core_cfg_t;

  localparam core_cfg_t RV32EMCCfg = '{
      ic        : IntController,
      num_irqs  : 64,
      num_prio  : 64,
      hart_id   : 0,
      boot_addr : BootAddr
  };

  localparam core_cfg_t DefaultCfg = RV32EMCCfg;

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
    addr_rule_t i2c;
    addr_rule_t tg;
    addr_rule_t cfg;
    addr_rule_t mtimer;
    addr_rule_t ext;
  } addr_map_t;

  localparam int unsigned ImemSize = `IMEM_BYTES;
  localparam int unsigned DmemSize = `DMEM_BYTES;

  localparam int unsigned TGSize = 4;

  localparam addr_rule_t DbgAddr = '{base : 32'h0000_0000, last : 32'h0000_1000};
  localparam addr_rule_t UartAddr = '{base : 32'h0000_3000, last : 32'h0000_3100};
  localparam addr_rule_t MtimerAddr = '{base : 32'h0000_3100, last : 32'h0000_3114};
  localparam addr_rule_t TimerGroupAddr = '{
      base : 32'h0000_3300,
      last : 32'h0000_3300 + (16 * TGSize)
  };
  localparam addr_rule_t I2cAddr = '{base : 32'h0000_3200, last : 32'h0000_3218};
  localparam addr_rule_t CfgAddr = '{base : 32'h0000_3500, last : 32'h0000_3600};
  localparam addr_rule_t ImemAddr = '{base : 32'h0001_0000, last : (32'h0001_0000 + ImemSize)};
  localparam addr_rule_t DmemAddr = '{base : 32'h0002_0000, last : (32'h0002_0000 + DmemSize)};
  localparam addr_rule_t ExtAddr = '{base : 32'h0003_0000, last : 32'h0010_0000};
  localparam addr_rule_t IntcAddr = '{base : 32'h0010_0000, last : 32'h0010_2000};

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
      i2c    : I2cAddr,
      cfg    : CfgAddr,
      tg     : TimerGroupAddr,
      mtimer : MtimerAddr,
      ext    : ExtAddr
  };


endpackage
