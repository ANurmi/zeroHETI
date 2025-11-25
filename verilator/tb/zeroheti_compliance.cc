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

  std::string SigPath ="";
  // ignore argv[0] (name of executable)
  for (int i=1; i<argc; i++) {
    const std::string arg_string(argv[i]);
    switch (arg_string[0]) {
      case '-':
        break;
      case '+':
        if(arg_string.substr(0,11) == "+signature=") {
          SigPath = arg_string.substr(11, arg_string.size());
        }
        break;
      default:
        break;
    }
  }

  Verilated::commandArgs(argc, argv);
  const std::string TracePath = ZhRoot + "/build/verilator_build/waveform.fst";
  std::cout << "[TB:init] Waveform path: " << TracePath << std::endl;
  ArchTestDriver<Vzeroheti_compliance>* drv = new ArchTestDriver<Vzeroheti_compliance>(TracePath);
  
  // TODO: take this from argv *if* needed
  const std::string ElfPath = "riscv.elf";

  load_memory(ElfPath, mem);
  
  // SIMULATION START
  drv->reset();
  drv->delay_cc(10);
  drv->drive(mem);

  delete drv;
  // SIMULATION END
  
  std::cout << "[TB:post] Exttracting signature file into " << SigPath << std::endl;
  parse_signature(mem, SigPath);

  return 0;
}
