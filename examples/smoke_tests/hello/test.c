#include "addr_map.h"
#include "mmio.h"
#include "uart.h"

int main() {

  init_uart(0x0, 0x0);

  for (int i=0; i<100; i++) asm("nop");
  write_serial('\n');
  print_uart("[UART] Hello from zeroHETI!\n");
  write_serial('\n');

  return 0;
}
