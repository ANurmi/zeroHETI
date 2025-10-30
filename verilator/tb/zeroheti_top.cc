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

  std::string elf_name = "";
  bool load_is_jtag = false;
  bool elf_given    = false;

  // ignore argv[0] (name of executable)
  for (int i=1; i<argc; i++) {
    const std::string arg_string(argv[i]);
    switch (arg_string[0]) {
      case '-':
        if (arg_string.substr(0,6) == "--load") {
          load_is_jtag = (arg_string.substr(6,5) == "=JTAG");
        } else {
          std::cout << "[Warning] unresolved - args found" << std::endl;
        }
        break;
      case '+':
        std::cout << "TODO: + args" << std::endl;
        break;
      default:
        elf_name = arg_string;
        elf_given = true;
        break;
    }
  }

  if (!elf_given) {
    std::cout << "[TB] No elf given, terminating simulation."
              << std::endl;
    std::exit(0);
  }
    /*
     * '--isa=rv32imc'
     * '+signature=/<>/riscof_work/src/add-01.S/dut/DUT-zeroheti.signature'
     * '+signature-granularity=4'
     * 'my.elf'
     *
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
      */

    //std::filesystem::path elfpath = "./test.elf";
    std::filesystem::path elfpath = tb->resolve_elf(elf_name);

    tb->open_trace("../build/verilator_build/waveform.fst");

    tb->reset();
    tb->jtag_reset_master();
    tb->jtag_init();

    if (load_is_jtag) {
      tb->jtag_run_elf(elfpath.string());
    } else {
      tb->jtag_halt_hart();
      std::cout << "[TB] Running preloaded program from entry point in " 
                << elfpath.string()
                << std::endl;
      tb->jtag_resume_hart_from(tb->get_entry(elfpath.string()));
    }
    tb->jtag_wait_eoc();
  

  delete tb;
  return 0;
}
