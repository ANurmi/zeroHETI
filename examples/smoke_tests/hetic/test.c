#include "addr_map.h"
#include "common.h"

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] Heterogeneous interrupt controller (HetIC) test\n");

  return 0;
}
