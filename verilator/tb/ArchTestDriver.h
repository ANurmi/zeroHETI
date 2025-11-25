const uint32_t CLOCK_PERIOD_PS = /* 100 MHz */ 10000;

typedef struct {
  bool req;
} mem_intf_t;

template <class VA>
class ArchTestDriver {
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

    void drive_instr_memory(std::unordered_map<uint32_t, uint32_t>& mem) {
      if (imem.req) {
        m_dut->instr_gnt_i    = 0;
        imem.req              = 0;
        m_dut->instr_rvalid_i = 1;
        m_dut->instr_rdata_i  = mem[m_dut->instr_addr_o];
      } else if (m_dut->instr_req_o) {
        m_dut->instr_gnt_i    = 1;
        imem.req              = 1;
        m_dut->instr_rvalid_i = 0;
      }
    }

    bool drive_data_memory(std::unordered_map<uint32_t, uint32_t>& mem) {
      if (dmem.req) {
        m_dut->data_gnt_i    = 0;
        dmem.req             = 0;
        m_dut->data_rvalid_i = 1;
        if (!m_dut->data_we_o) { // read
          m_dut->data_rdata_i = mem[m_dut->data_addr_o];
        }
      } else {
        if (m_dut->data_req_o) {
          if (m_dut->data_we_o) { // write
            mem[m_dut->data_addr_o] = m_dut->data_wdata_o;
            //printf("Wrote %08X to %08X\n",
            //  m_dut->data_wdata_o, m_dut->data_addr_o);
          }
          m_dut->data_gnt_i    = 1;
          dmem.req             = 1;
        }
        m_dut->data_rvalid_i = 0;
      }
      return false;
    }

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

    bool drive(std::unordered_map<uint32_t, uint32_t>& mem) {
      bool got_exit;
      for (int i=0; i<500; i++){
        drive_instr_memory(mem);
        got_exit = drive_data_memory(mem);
        tick();
      }
      std::cout << got_exit << std::endl;
      return true;
    }

};

