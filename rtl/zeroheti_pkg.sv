package zeroheti_pkg;

  typedef struct packed {
    bit rve;
    bit bt_alu;
    bit wb_stage;
    ibex_pkg::rv32m_e mul;
    int unsigned num_irqs;
    int unsigned num_prio;
    int unsigned hart_id;
    logic [31:0] boot_addr;
  } core_cfg_t;


  localparam core_cfg_t Rv32EcCfg = '{
      rve       : 1,
      bt_alu    : 0,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MNone,
      num_irqs  : 32,
      num_prio  : 32,
      hart_id   : 0,
      boot_addr : 32'h0800
  };

  localparam core_cfg_t Rv32EmcCfg = '{
      rve       : 1,
      bt_alu    : 1,
      wb_stage  : 1,
      mul       : ibex_pkg::RV32MFast,
      num_irqs  : 32,
      num_prio  : 32,
      hart_id   : 0,
      boot_addr : 32'h0800
  };

  localparam core_cfg_t Rv32IcCfg = '{
      rve       : 0,
      bt_alu    : 0,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MNone,
      num_irqs  : 32,
      num_prio  : 32,
      hart_id   : 0,
      boot_addr : 32'h0800
  };

  localparam core_cfg_t Rv32ImcCfg = '{
      rve       : 0,
      bt_alu    : 1,
      wb_stage  : 1,
      mul       : ibex_pkg::RV32MSingleCycle,
      num_irqs  : 32,
      num_prio  : 32,
      hart_id   : 0,
      boot_addr : 32'h0800
  };

  localparam core_cfg_t DefaultCfg = Rv32EcCfg;

  typedef struct packed {
    logic [31:0] base;
    logic [31:0] last;
  } addr_rule_t;

  typedef struct packed {
    addr_rule_t dbg;
    addr_rule_t imem;
    addr_rule_t dmem;
    addr_rule_t hetic;
    addr_rule_t uart;
    addr_rule_t i2c;
    addr_rule_t tg;
    addr_rule_t mtimer;
    addr_rule_t ext;
  } addr_map_t;

  localparam int unsigned ImemSize = `IMEM_BYTES;
  localparam int unsigned DmemSize = `DMEM_BYTES;

  localparam int unsigned TGSize = 4;

  localparam addr_rule_t DbgAddr = '{base : 32'h0000_0000, last : 32'h0000_1000};
  localparam addr_rule_t HetIcAddr = '{base : 32'h0000_1000, last : 32'h0000_2000};
  localparam addr_rule_t UartAddr = '{base : 32'h0000_2000, last : 32'h0000_2100};
  localparam addr_rule_t MtimerAddr = '{base : 32'h0000_2100, last : 32'h0000_2114};
  localparam addr_rule_t TimerGroupAddr = '{
      base : 32'h0000_2114,
      last : 32'h0000_2114 + (12 * TGSize)
  };
  localparam addr_rule_t I2cAddr = '{base : 32'h0000_2200, last : 32'h0000_2218};
  localparam addr_rule_t ImemAddr = '{base : 32'h0001_0000, last : (32'h0001_0000 + ImemSize)};
  localparam addr_rule_t DmemAddr = '{base : 32'h0002_0000, last : (32'h0002_0000 + DmemSize)};
  localparam addr_rule_t ExtAddr = '{base : 32'h0003_0000, last : 32'hFFFF_FFFF};

  // imem size in words
  localparam int unsigned ImemWSize = ImemSize / 4;
  // dmem size in words
  localparam int unsigned DmemWSize = DmemSize / 4;

  localparam addr_map_t AddrMap = '{
      dbg    : DbgAddr,
      imem   : ImemAddr,
      dmem   : DmemAddr,
      hetic  : HetIcAddr,
      uart   : UartAddr,
      i2c    : I2cAddr,
      tg     : TimerGroupAddr,
      mtimer : MtimerAddr,
      ext    : ExtAddr
  };


endpackage
