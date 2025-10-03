#include "addr_map.h"
#include "common.h"

int main() {

  init_uart(0x0, 0x0);

  for (int i=0; i<100; i++) asm("nop");
  print_uart("[UART] zeroHETI mtime test\n");

  write_u32(MTIMER_CTRL, 0x1);

  uint32_t time = read_u32(MTIMER_TIME_LO);
  
  if (time > 0) {
    print_uart("[UART] MTIMER is ticking!\n");
  } else {
    print_uart("[UART] MTIMER is NOT ticking!\n");
  }
  return 0;
}
