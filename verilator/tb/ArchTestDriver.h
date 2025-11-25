const uint32_t CLOCK_PERIOD_PS = /* 100 MHz */ 10000;

typedef struct {
  bool req;
  uint32_t addr;
  uint32_t wdata;
  uint8_t be;
} mem_intf_t;

template <class VA>
class ArchTestDriver {
  public:

    ArchTestDriver(const std::string TracePath) : m_tickcount(0) {
      Verilated::traceEverOn(true);
      m_dut   = new VA;
      imem    = {};
      dmem    = {};
      open_trace(TracePath.c_str());
      m_dut->clk_i = 0;
      m_dut->eval();
    }

    ~ArchTestDriver(void){
      close_trace();
      delete m_dut;
      m_dut = NULL;
    }

    void reset(void) {
      m_dut->rst_ni = 0;
      tick();
      m_dut->rst_ni = 1;
    }

    void delay_cc(uint32_t cycles) {
      for (uint32_t i=0; i<cycles;i++) tick();
    }

    void drive(std::unordered_map<uint32_t, uint32_t>& mem) {
      bool got_exit;
      while(!check_exit()) {
        drive_instr_obi(mem);
        drive_data_obi(mem);
        tick();
      }
    }

  private:
    VA              *m_dut;
    VerilatedFstC*   m_trace;
    uint64_t         m_tickcount;
    // memory interface state flags
    mem_intf_t       imem;
    mem_intf_t       dmem;

    void open_trace(const char* fst_name){
      if (!m_trace){
        m_trace = new VerilatedFstC;
        m_dut->trace(m_trace, 99); // 99 levels of hierarchy
        m_trace->open(fst_name);
      }
    }

    void close_trace(void) {
      if (m_trace) {
        m_trace->close();
        delete m_trace;
        m_trace = NULL;
      }
    }
    void tick(void) {
        m_tickcount++;
        m_dut->eval();
        if (m_trace) m_trace->dump((vluint64_t)(CLOCK_PERIOD_PS*m_tickcount-CLOCK_PERIOD_PS/5));
        m_dut->clk_i = 1;
        m_dut->eval();
        if (m_trace) m_trace->dump((vluint64_t)(CLOCK_PERIOD_PS*m_tickcount));
        m_dut->clk_i = 0;
        m_dut->eval();
        if (m_trace){
            m_trace->dump((vluint64_t)(CLOCK_PERIOD_PS*m_tickcount+CLOCK_PERIOD_PS/2));
            m_trace->flush();
        }
    }

    bool check_exit(void) {
      return m_dut->data_req_o &
            (m_dut->data_wdata_o == 0x80000000) &
            (m_dut->data_addr_o  == 0x00000380) &
             m_dut->data_we_o;
    }

    void drive_instr_obi(std::unordered_map<uint32_t, uint32_t>& mem) {
      // "default" assignments
      m_dut->instr_rvalid_i = 0;
      m_dut->instr_gnt_i    = 0;
      
      if (m_dut->instr_req_o & !imem.req){
        m_dut->instr_gnt_i = 1;
        imem.req  = 1;
        imem.addr = m_dut->instr_addr_o;
      } else {
        if (imem.req) {
          m_dut->instr_rvalid_i = 1;
          m_dut->instr_rdata_i  = mem[imem.addr];
        }
        imem.req           = 0;
        m_dut->instr_gnt_i = 0;
      }
    }

    void drive_data_obi(std::unordered_map<uint32_t, uint32_t>& mem) {
      // "default" assignments
      m_dut->data_rvalid_i = 0;
      m_dut->data_gnt_i    = 0;

      if (m_dut->data_req_o & !dmem.req){
        m_dut->data_gnt_i    = 1;
        dmem.req             = 1;
        dmem.addr            = m_dut->data_addr_o;
        dmem.wdata           = m_dut->data_wdata_o;
        dmem.be              = m_dut->data_be_o;
      } else {
        if (dmem.req) {
          m_dut->data_rvalid_i = 1;
          if (!m_dut->data_we_o) {
            m_dut->data_rdata_i  = mem[dmem.addr];
          } else {
            uint32_t value = mem[dmem.addr];
            for (int i=0; i<4; i++) {
              if (dmem.be & 1 << i) {
                // clear old bits
                value &= ~(0xff << i*8);
                // set new bits
                uint8_t mask = (uint8_t)(dmem.wdata >> i*8);
                value |= ((uint32_t) mask) << i*8;
              }
            }
            mem[dmem.addr] = value;
          }
        }
        dmem.req             = 0;
        m_dut->data_gnt_i    = 0;
      }
    }
};

