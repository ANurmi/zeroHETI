#include "addr_map.h"
#include "mmio.h"
#include "uart.h"

int main() {

  init_uart(100000000, 3000000);

  print_uart("[UART] Hello from zeroHETI!\n");
  print_uart("[UART] more elaborate prints to exercise the UART\n");

  return 0;
}
