package zeroheti_pkg;

typedef struct packed {
  bit rve;
  bit bt_alu;
  bit wb_stage;
  int unsigned num_irqs;
  int unsigned hart_id;
  logic [31:0] boot_addr;
} core_cfg_t;

localparam core_cfg_t DefaultCfg = '{
  rve       : 1,
  bt_alu    : 0,
  wb_stage  : 0,
  num_irqs  : 32,
  hart_id   : 0,
  boot_addr : 32'h0800
};

typedef struct packed {
  logic [31:0] base;
  logic [31:0] last;
} addr_rule_t;

typedef struct packed {
  addr_rule_t dbg;
  addr_rule_t imem;
  addr_rule_t dmem;
  addr_rule_t zhic;
  addr_rule_t uart;
  addr_rule_t mtimer;
  addr_rule_t ext;
} addr_map_t;

localparam addr_rule_t DbgAddr    = '{ base : 32'h0000_0000, last : 32'h0000_1000 };
localparam addr_rule_t ImemAddr   = '{ base : 32'h0000_1000, last : 32'h0000_5000 };
localparam addr_rule_t DmemAddr   = '{ base : 32'h0000_5000, last : 32'h0000_9000 };
localparam addr_rule_t ZhicAddr   = '{ base : 32'h0000_9000, last : 32'h0000_A000 };
localparam addr_rule_t UartAddr   = '{ base : 32'h0000_A000, last : 32'h0000_A100 };
localparam addr_rule_t MtimerAddr = '{ base : 32'h0000_A100, last : 32'h0000_A114 };
localparam addr_rule_t ExtAddr    = '{ base : 32'h0001_0000, last : 32'hFFFF_FFFF };

localparam addr_map_t AddrMap = '{
  dbg    : DbgAddr,
  imem   : ImemAddr,
  dmem   : DmemAddr,
  zhic   : ZhicAddr,
  uart   : UartAddr,
  mtimer : MtimerAddr,
  ext    : ExtAddr
};


endpackage
