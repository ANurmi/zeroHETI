#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"
#include "mtimer.h"
#include "i2c.h"

#define WRITE    1
#define LAST     1
#define NOT_LAST 0
#define READ     0

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] PID-control demonstrator on i2c\n");
  
  /* GLOBAL SETUP */
  i2c_set_prescaler(4);
  i2c_core_enable();

  // MTimer to terminate test

  // 1  s in hex @100MHz = 0x05F5_E100
  // 1 ms,               = 0x0001_86A0
  // 1 us,               = 0x0000_0064
  set_mtimer_cmp(0x186A0);
  start_mtimer();

  // Writing 1 to address 0 activates
  // power control simulation 
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x1, LAST);
  /* START CRITICAL SECTION*/
  for(int i=0;i<1000;i++) asm("nop");
  i2c_send_addr_frame(4, WRITE);
  i2c_send_data_frame(0x22, LAST);
  while(!get_ip(IRQ_IDX_MTIME));
  /* END CRITICAL SECTION*/
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x0, LAST);
  print_uart("[UART] Got MTIME irq\n");
  
  return 0;
}
