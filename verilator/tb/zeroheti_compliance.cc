#include <iostream>
#include <fstream>
#include <filesystem>

#define xstr(s) str(s)
#define str(s) #s

const std::string ZhRoot = xstr(ZH_ROOT);
const std::string Canary = "6f5ca309";

#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_compliance.h"
#include "ArchTestDriver.h"

int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);

  const std::string TracePath = ZhRoot + "/build/verilator_build/waveform.fst";
  std::cout << "[TB] Waveform path: " << TracePath << std::endl;
  ArchTestDriver<Vzeroheti_compliance>* drv = new ArchTestDriver<Vzeroheti_compliance>(TracePath);

  // TODO: taket this from argv
  const std::string SigPath   = ZhRoot + "/build/verilator_build/test.signature";
  //tb->open_trace(TracePath.c_str());

  //tb->reset();
  drv->reset();

  drv->delay(100);
  
  std::cout << "TODO: parse dump into sig file" << std::endl;
  parse_signature(SigPath);

  delete drv;
  return 0;
}
