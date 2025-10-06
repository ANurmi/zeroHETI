#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] Heterogeneous interrupt controller (HetIC) test\n");
  
  set_ip(0);
  set_ip(1);
  set_ip(5);

  clear_ip(1);

  set_prio(0, 0x11);
  set_ie(0);

  return 0;
}
