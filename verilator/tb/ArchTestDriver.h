const uint32_t CLOCK_PERIOD_PS = /* 100 MHz */ 10000;

template <class VA>
class ArchTestDriver {
  private:
    VA              *m_dut;
    VerilatedFstC*   m_trace;
    uint64_t         m_tickcount;

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

    void drive_instr_memory(void) {

    }

    void drive_data_memory(void) {
    }

  public:

    ArchTestDriver(const std::string TracePath) : m_tickcount(0) {
      Verilated::traceEverOn(true);
      m_dut   = new VA;
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

    bool drive() {
      drive_instr_memory();
      drive_data_memory();
      tick();
      return true;
    }

};

