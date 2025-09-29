package zeroheti_pkg;

typedef struct packed {
  bit rve;
  bit bt_alu;
  bit wb_stage;
  int unsigned num_irqs;
} core_cfg_t;

typedef struct packed {
  logic [31:0] base;
  logic [31:0] last;
} addr_rule_t;

typedef struct packed {
  addr_rule_t clic;
} addr_map_t;

localparam addr_rule_t ClicAddr = '{
  base : 32'h9000,
  last : 32'hA000
};

localparam addr_map_t AddrMap = '{
  clic : ClicAddr
};

localparam core_cfg_t DefaultCfg = '{
  rve      : 1,
  bt_alu   : 0,
  wb_stage : 0,
  num_irqs : 32
};

endpackage
