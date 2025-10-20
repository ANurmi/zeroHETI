#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"

static uint8_t isr_flag = 1;

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] sw_irq test\n");

  set_ip(3);
  set_ie(3);
  set_prio(3, 14);
  interrupts_enable();

  // Spin to wait for isr
  for (int i=0;i<100; i++) asm("nop");
  
return isr_flag;
}

void machine_soft_isr(void) {
  print_uart("[UART] entered succesfully");
  isr_flag = 0;
}
