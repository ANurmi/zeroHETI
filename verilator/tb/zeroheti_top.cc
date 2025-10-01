#include <iostream>

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

  tb->open_trace("../build/verilator_build/waveform.fst");
  tb->print_logo();

  for (int it=0;it<100;it++) tb->tick();

  tb->reset();
  tb->jtag_reset_master();
  tb->jtag_init();
  tb->jtag_halt_hart();
  tb->jtag_resume_hart_from(0x1000);
  for (int it=0;it<100;it++) tb->tick();
 // tb->jtag_wait_eoc();

  delete tb;

  return 0;
}
