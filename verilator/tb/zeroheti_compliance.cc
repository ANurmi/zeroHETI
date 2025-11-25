#include <iostream>
#include <fstream>
#include <filesystem>
#include <unordered_map>

#define xstr(s) STR(s)
#define STR(s) #s

const std::string ZhRoot = xstr(ZH_ROOT);

#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_compliance.h"
#include "ArchTestDriver.h"
#include "TbUtils.h"

// Model the memory as a dynamic associative array
std::unordered_map<uint32_t, uint32_t> mem;

int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);
  const std::string TracePath = ZhRoot + "/build/verilator_build/waveform.fst";
  std::cout << "[TB:init] Waveform path: " << TracePath << std::endl;
  ArchTestDriver<Vzeroheti_compliance>* drv = new ArchTestDriver<Vzeroheti_compliance>(TracePath);
  
  // TODO: taket this from argv if needed
  const std::string ElfPath = "riscv.elf";

  load_memory(ElfPath, mem);
  
  // SIMULATION START
  // TODO: taket this from argv
  const std::string SigPath   = ZhRoot + "/build/verilator_build/test.signature";

  drv->reset();
  drv->delay_cc(10);
  drv->drive(mem);

  delete drv;
  // SIMULATION END
  
  std::cout << "TODO: parse dump into sig file" << std::endl;
  std::cout << "[TB:post] Exttracting signature file" << std::endl;
  parse_signature(mem, SigPath);

  return 0;
}
