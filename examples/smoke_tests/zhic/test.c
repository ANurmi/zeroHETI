#include "addr_map.h"
#include "common.h"

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] zeroHETI interrupt controller (ZHIC) test\n");

  return 0;
}
