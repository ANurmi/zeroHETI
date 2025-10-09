#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "mtimer.h"
#include "hetic.h"

#define EDGE  0
#define LEVEL 1

#define POS   0
#define NEG   1


int main() {

  init_uart(0x0, 0x0);

  for (int i=0; i<100; i++) asm("nop");
  print_uart("[UART] zeroHETI mtime test\n");

  //set_trig_type(7, LEVEL);
  //set_trig_type(7, EDGE);

  //set_trig_pol(7, NEG);
  //set_trig_pol(7, POS);

  set_mtimer_cmp(50);
  set_mtimer_prescaler(0x1);
  set_mtimer_prescaler(0x3);
  start_mtimer();
  for (int i=0; i<100; i++) asm("nop");
  
  return 0;
}
