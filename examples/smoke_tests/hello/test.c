#include "addr_map.h"
#include "common.h"

int main() {

  init_uart(0x0, 0x0);

  for (int i=0; i<100; i++) asm("nop");
  print_uart("[UART] Hello from zeroHETI!\n");
  write_serial('\n');

  return 0;
}
