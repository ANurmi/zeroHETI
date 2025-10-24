#include <iostream>
#include <filesystem>

#define CLKI      clk_i
#define RSTNI     rst_ni

#define JTAGTMS   jtag_tms_i
#define JTAGTCK   jtag_tck_i
#define JTAGTDI   jtag_td_i
#define JTAGTDO   jtag_td_o
#define JTAGTRSTN jtag_trst_ni

#define IDCODE    0xfeedc0d3

#include "zeroheti_top.h"


int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);

  TbZeroHeti* tb = new TbZeroHeti();
  tb->print_logo();

  if (argc == 1) {
    printf("[TB] No TEST specified, exiting..\n\n");
  } else {

    std::cout << "[TB] args: " << argc << std::endl;
    /*
    const std::string Elf(argv[1]);
    std::cout << "[TB] Looking for ELF: " << Elf << std::endl;
    std::filesystem::path elfpath = std::string("../build/sw/") + Elf + ".elf";
    bool elfPathExists = std::filesystem::exists(elfpath);

    const std::string Load(argv[2]);
    std::cout << "[TB] Load: " << Load << std::endl;
    
    if(!elfPathExists) {
      std::cout << "[TB] ERROR! ELF not found in path " << elfpath << std::endl << std::endl;
    } else {
      std::cout << "[TB] ELF path is " << elfpath << std::endl;
      tb->open_trace("../build/verilator_build/waveform.fst");

      tb->reset();
      tb->jtag_reset_master();
      tb->jtag_init();

      if (Load == "JTAG") {
        tb->jtag_run_elf(elfpath.string());
      } else {
        tb->jtag_halt_hart();
        std::cout << "[TB] Running elaboration-time preloaded program from entry point in " 
                  << elfpath.string()
                  << std::endl;
        tb->jtag_resume_hart_from(tb->get_entry(elfpath.string()));
      }
      tb->jtag_wait_eoc();
    }
  dd*/
  }

  delete tb;
  return 0;
}
