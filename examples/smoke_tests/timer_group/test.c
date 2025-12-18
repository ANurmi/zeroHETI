#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "mtimer.h"
#include "hetic.h"

int main() {

  init_uart(100000000, 3000000);

  print_uart("[UART] zeroHETI timer group sanity test\n");

  print_uart("[UART] setting TimerCmp0\n");
  /*set_mtimer_cmp(50);
  set_mtimer_prescaler(0x1);
  set_mtimer_prescaler(0x3);
  start_mtimer();*/
  
  /*while(!get_ip(MTIME_IDX)) {
    print_uart("[UART] polling for mtimer.ip\n");
  }*/
  //print_uart("[UART] IRQ 7 pended by machine timer (mtimer)\n");
  
  return 0;
}
