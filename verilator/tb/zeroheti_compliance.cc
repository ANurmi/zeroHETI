#include <iostream>
#include <fstream>
#include <filesystem>

#define CLKI      clk_i
#define RSTNI     rst_ni

#define JTAGTMS   jtag_tms_i
#define JTAGTCK   jtag_tck_i
#define JTAGTDI   jtag_td_i
#define JTAGTDO   jtag_td_o
#define JTAGTRSTN jtag_trst_ni

#define IDCODE    0xfeedc0d3
#define xstr(s) str(s)
#define str(s) #s

const std::string ZhRoot = xstr(ZH_ROOT);
const std::string Canary = "6f5ca309";

#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_compliance.h"
#include "zeroheti_compliance.h"
#include "Testbench.h"


int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);

  Testbench<Vzeroheti_compliance>* tb = new Testbench<Vzeroheti_compliance>;
  const std::string TracePath = ZhRoot + "/build/verilator_build/waveform.fst";
  // TODO: taket this from argv
  const std::string SigPath   = ZhRoot + "/build/verilator_build/test.signature";
  std::cout << "[TB] Waveform path: " << TracePath << std::endl;
  tb->open_trace(TracePath.c_str());

  tb->reset();

  //while(!tb->m_dut->test_done_o) 
  tb->tick();
  tb->tick();
  tb->tick();
  tb->tick();
  
  std::cout << "TODO: parse dump into sig file" << std::endl;
  parse_signature(SigPath);

  delete tb;
  return 0;
}
