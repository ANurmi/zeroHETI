#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"
#include "i2c.h"

#define START 1
#define NO_START 0
#define STOP 1
#define NO_STOP 0
//#define IRQ 1
//#define NO_IRQ 0

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] zeroHETI i2c test\n");
  
  // extra cycles to stabilize prints
  for (int i=0; i<100; i++) asm("nop");  

  i2c_set_prescaler(2);
  i2c_core_enable();
  //i2c_irq_enable();  

  i2c_write_msg(67, START, NO_STOP);
  i2c_write_msg(43, NO_START, NO_STOP);
  i2c_read_msg(NO_START, NO_STOP);
  i2c_read_msg(NO_START, NO_STOP);
  i2c_write_msg(11, NO_START, NO_STOP);
  i2c_write_msg(99, NO_START, STOP);

  // extra cycles to stabilize prints
  for (int i=0; i<50; i++) asm("nop");  
  return 0;
}
