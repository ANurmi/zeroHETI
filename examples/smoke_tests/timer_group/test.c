#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"
#include "timer_group.h"

int main() {

  init_uart(100000000, 3000000);

  print_uart("[UART] zeroHETI timer group sanity test\n");

  print_uart("[UART] Testing TimerCmp0 irq\n");
  timer_group_set_cmp(0, 0x123);
  timer_group_start(0);
  while(!get_ip(IRQ_IDX_TG0_CMP));

  print_uart("[UART] Testing TimerCmp1 irq\n");
  timer_group_set_cmp(1, 0x223);
  timer_group_start(1);
  while(!get_ip(IRQ_IDX_TG1_CMP));

  print_uart("[UART] Testing TimerCmp2 irq\n");
  timer_group_set_cmp(2, 0x10);
  timer_group_start(2);
  while(!get_ip(IRQ_IDX_TG2_CMP));

  print_uart("[UART] Testing TimerCmp3 irq\n");
  timer_group_set_cmp(3, 0x100);
  timer_group_start(3);
  while(!get_ip(IRQ_IDX_TG3_CMP));
  
  return 0;
}
