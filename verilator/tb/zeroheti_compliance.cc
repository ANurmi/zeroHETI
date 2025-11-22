#include <iostream>
#include <fstream>
#include <filesystem>
#include <unordered_map>

#define xstr(s) str(s)
#define str(s) #s

const std::string ZhRoot = xstr(ZH_ROOT);
const std::string Canary = "6f5ca309";

#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_compliance.h"
#include "ArchTestDriver.h"

// Model the memory as a dynamic associative array
std::unordered_map<uint32_t, uint32_t> mem;

int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);
  const std::string TracePath = ZhRoot + "/build/verilator_build/waveform.fst";
  std::cout << "[TB] Waveform path: " << TracePath << std::endl;
  ArchTestDriver<Vzeroheti_compliance>* drv = new ArchTestDriver<Vzeroheti_compliance>(TracePath);
  
  // SIMULATION START
  // TODO: taket this from argv
  const std::string SigPath   = ZhRoot + "/build/verilator_build/test.signature";

  drv->reset();
  drv->delay_cc(10);

  bool signal_test_end = false;
  while (!signal_test_end) {
    signal_test_end = drv->drive();
  }

  delete drv;
  // SIMULATION END

  std::cout << "TODO: parse dump into sig file" << std::endl;
  parse_signature(SigPath);

  return 0;
}
