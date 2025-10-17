#include "addr_map.h"
#include "mmio.h"
#include "uart.h"

int main() {

  //init_uart(100000000, 3000000);
  //TODO: Fix compiler ABI issue
  init_uart(0, 0);

  for (int i=0; i<100; i++) asm("nop");
  write_serial('\n');
  print_uart("[UART] Hello from zeroHETI!\n");
  write_serial('\n');

  return 0;
}
