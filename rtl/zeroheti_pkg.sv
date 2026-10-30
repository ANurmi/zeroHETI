package zeroheti_pkg;

  typedef enum integer {
    CLIC  = 1,
    EDFIC = 2
  } int_ctrl_e;

  typedef struct packed {
    int_ctrl_e   intc;
    int unsigned num_src;
    int unsigned num_prio;
  } irq_t;

  typedef struct packed {
    bit rve;
    bit bt_alu;
    bit wb_stage;
    ibex_pkg::rv32m_e mul;
    ibex_pkg::regfile_e rf;
    int unsigned hart_id;
  } core_cfg_t;

  typedef struct packed {
    int unsigned bytes;
    int unsigned banks;
    bit          lp;
  } mem_t;

  localparam core_cfg_t MinCoreCfg = '{
      rve       : 1,
      bt_alu    : 0,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MNone,
      rf        : ibex_pkg::RegFileFF,
      hart_id   : 0
  };

  localparam core_cfg_t DefaultCoreCfg = '{
      rve       : 1,
      bt_alu    : 1,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MSingleCycle,
      rf        : ibex_pkg::RegFileFF,
      hart_id   : 0
  };

  localparam core_cfg_t BpssAsicCfg = '{
      rve       : 1,
      bt_alu    : 1,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MNone,
      rf        : ibex_pkg::RegFileLatch,
      hart_id   : 0
  };

  localparam core_cfg_t CpssAsicCfg = '{
      rve       : 1,
      bt_alu    : 1,
      wb_stage  : 0,
      mul       : ibex_pkg::RV32MSingleCycle,
      rf        : ibex_pkg::RegFileLatch,
      hart_id   : 0
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
    addr_rule_t tq;
    addr_rule_t rom;
    addr_rule_t cfg;
    addr_rule_t mtimer;
    addr_rule_t mbx;
    addr_rule_t ext;
  } addr_map_t;

  function static logic [31:0] get_end_addr(input logic [31:0] base, input logic [31:0] size);
    return base + size;
  endfunction

  localparam addr_rule_t DbgAddr = '{base : 32'h0000_0000, last : 32'h0000_1000};
  /* 0x0000_1000 - 0x0001_0000 reserved*/
  localparam addr_rule_t ImemAddr = '{base : 32'h0001_0000, last : 32'h0002_0000};  // variable
  localparam addr_rule_t DmemAddr = '{base : 32'h0002_0000, last : 32'h0003_0000};  // variable
  localparam addr_rule_t RomAddr = '{base : 32'h0003_0000, last : 32'h0003_3000};
  localparam addr_rule_t UartAddr = '{base : 32'h0003_3000, last : 32'h0003_3100};
  localparam addr_rule_t MtimerAddr = '{base : 32'h0003_3100, last : 32'h0003_3114};
  localparam addr_rule_t I2c0Addr = '{base : 32'h0003_3200, last : 32'h0003_3300};
  localparam addr_rule_t I2c1Addr = '{base : 32'h0003_3300, last : 32'h0003_3400};
  localparam addr_rule_t TimerGroupAddr = '{base : 32'h0003_3400, last : 32'h0003_4000};  //variable
  localparam addr_rule_t CfgAddr = '{base : 32'h0003_4000, last : 32'h0003_5000};
  localparam addr_rule_t TqAddr = '{base: 32'h0003_3800, last: 32'h0003_3900};
  localparam addr_rule_t SpiAddr = '{base : 32'h0003_5000, last : 32'h0003_5100};
  localparam addr_rule_t IntcAddr = '{base : 32'h0010_0000, last : 32'h0010_2000};
  localparam addr_rule_t MbxAddr = '{base : 32'h0010_8000, last : 32'h0010_8100};
  localparam addr_rule_t ExtAddr = '{base : 32'h0011_0000, last : 32'hFFFF_FFFF};


  localparam addr_map_t ZHAddrMap = '{
      dbg    : DbgAddr,
      imem   : ImemAddr,
      dmem   : DmemAddr,
      intc   : IntcAddr,
      uart   : UartAddr,
      i2c_0  : I2c0Addr,
      i2c_1  : I2c1Addr,
      cfg    : CfgAddr,
      rom    : RomAddr,
      spi    : SpiAddr,
      tg     : TimerGroupAddr,
      tq     : TqAddr,
      mtimer : MtimerAddr,
      mbx    : MbxAddr,
      ext    : ExtAddr
  };

  localparam addr_map_t BpAddrMap = '{
      dbg    : DbgAddr,
      imem   : ImemAddr,
      dmem   : DmemAddr,
      intc   : IntcAddr,
      uart   : UartAddr,
      cfg    : CfgAddr,
      rom    : RomAddr,
      spi    : SpiAddr,
      mtimer : MtimerAddr,
      ext    : ExtAddr,
      default: 0
  };

  localparam addr_map_t PpAddrMap = '{dbg    : DbgAddr, default: 0};

  typedef struct packed {
    mem_t        imem;
    mem_t        dmem;
    int unsigned size_tg;
    irq_t        irq;
    core_cfg_t   core;
    addr_map_t   am;
    logic [31:0] jtag_idcode;
  } platform_cfg_t;

  localparam mem_t BpssMem = '{bytes: 32768, banks: 4, lp: 1};
  localparam mem_t CpssIMem = '{bytes: 65536, banks: 8, lp: 0};
  localparam mem_t CpssDMem = '{bytes: 32768, banks: 4, lp: 0};
  localparam mem_t PpssIMem = '{bytes: 1024, banks: 1, lp: 0};
  localparam mem_t PpssDMem = '{bytes: 1024, banks: 1, lp: 0};

  localparam irq_t IrqMin = '{intc: CLIC, num_src : 32, num_prio: 8};
  localparam irq_t IrqClic = '{intc: CLIC, num_src : 128, num_prio: 128};
  localparam irq_t IrqEdfic = '{intc: EDFIC, num_src : 128, num_prio: 128};

  localparam platform_cfg_t BpssCfg = '{
      imem: BpssMem,
      dmem: BpssMem,
      size_tg : 0,
      core: BpssAsicCfg,
      irq: IrqMin,
      am: BpAddrMap,
      jtag_idcode : 32'h0BAD_C0D3
  };
  localparam platform_cfg_t CpssCfg = '{
      imem: CpssIMem,
      dmem: CpssIMem,
      size_tg : 16,
      core: CpssAsicCfg,
      irq: IrqEdfic,
      am: ZHAddrMap,
      jtag_idcode: 32'hFEED_C0D3
  };
  localparam platform_cfg_t PpssCfg = '{
      imem: PpssIMem,
      dmem: PpssDMem,
      size_tg : 0,
      core: CpssAsicCfg,
      irq: IrqMin,
      am: PpAddrMap,
      jtag_idcode: 32'hF007_BA11
  };
  localparam platform_cfg_t EDFICCfg = '{
      imem: CpssIMem,
      dmem: CpssIMem,
      size_tg    : 16,
      core        : DefaultCoreCfg,
      irq: IrqEdfic,
      am: ZHAddrMap,
      jtag_idcode: 32'hFEED_C0D3
  };
  localparam platform_cfg_t CLICCfg = '{
      imem: CpssIMem,
      dmem: CpssIMem,
      size_tg : 16,
      core: DefaultCoreCfg,
      irq: IrqClic,
      am: ZHAddrMap,
      jtag_idcode: 32'hFEED_C0D3
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
