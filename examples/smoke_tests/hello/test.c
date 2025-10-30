#include "addr_map.h"
#include "mmio.h"
#include "uart.h"

int main() {

  //init_uart(100000000, 3000000);
  //TODO: Fix compiler ABI issue
  init_uart(0, 0);

  write_serial('\n');
  print_uart("[UART] Hello from zeroHETI!\n");
  print_uart("[UART] Bottom Text\n");
  write_serial('\n');

  return 0;
}
