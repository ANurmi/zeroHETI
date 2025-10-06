#include "addr_map.h"
#include "mmio.h"
#include "uart.h"

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] Heterogeneous interrupt controller (HetIC) test\n");

  write_u16(HETIC_BASE, 0xFFFF);

  return 0;
}
