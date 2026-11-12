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
      wb_stage  : 1,
      mul       : ibex_pkg::RV32MSingleCycle,
      rf        : ibex_pkg::RegFileFF,
      hart_id   : 0
  };

  typedef struct packed {
    mem_t                                imem;
    mem_t                                dmem;
    int unsigned                         size_tg;
    irq_t                                irq;
    core_cfg_t                           core;
    zeroheti_address_map_pkg::addr_map_t am;
    logic [31:0]                         jtag_idcode;
  } platform_cfg_t;

  localparam zeroheti_address_map_pkg::addr_map_t AddrMap = zeroheti_address_map_pkg::AddressMap;

  localparam int unsigned SizeImem = AddrMap.core.imem.last - AddrMap.core.imem.base;
  localparam int unsigned SizeDmem = AddrMap.core.dmem.last - AddrMap.core.dmem.base;

  localparam mem_t Imem = '{bytes: SizeImem, banks: 8};
  localparam mem_t Dmem = '{bytes: SizeDmem, banks: 8};

  localparam irq_t IrqMin = '{intc: CLIC, num_src : 32, num_prio: 8};
  localparam irq_t IrqClic = '{intc: CLIC, num_src : 128, num_prio: 128};
  localparam irq_t IrqEdfic = '{intc: EDFIC, num_src : 128, num_prio: 128};

  localparam platform_cfg_t EDFICCfg = '{
      imem: Imem,
      dmem: Dmem,
      size_tg    : 16,
      core        : DefaultCoreCfg,
      irq: IrqEdfic,
      am: AddrMap,
      jtag_idcode: 32'hFEED_C0D3
  };
  localparam platform_cfg_t CLICCfg = '{
      imem: Imem,
      dmem: Dmem,
      size_tg : 16,
      core: DefaultCoreCfg,
      irq: IrqClic,
      am: AddrMap,
      jtag_idcode: 32'hFEED_C0D3
  };

endpackage
