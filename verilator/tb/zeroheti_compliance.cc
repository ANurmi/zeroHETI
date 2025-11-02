#include <iostream>
#include <filesystem>

#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_compliance.h"


int main(int argc, char** argv) {

  Verilated::commandArgs(argc, argv);

  Vzeroheti_compliance* tb = new Vzeroheti_compliance();
  printf("zh compliance\n");
/*
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
    / *
     * '--isa=rv32imc'
     * '+signature=/<>/riscof_work/src/add-01.S/dut/DUT-zeroheti.signature'
     * '+signature-granularity=4'
     * 'my.elf'
     * /

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
  */

  delete tb;
  return 0;
}
