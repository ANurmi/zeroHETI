#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "hetic.h"
#include "i2c.h"

#define WRITE    1
#define LAST     1
#define NOT_LAST 0
#define READ     0

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] zeroHETI i2c test\n");
  
  // extra cycles to stabilize prints
  for (int i=0; i<100; i++) asm("nop");  

  i2c_set_prescaler(4);
  i2c_core_enable();

  i2c_send_addr_frame(8, READ);
  uint8_t test = i2c_recv_data_frame(LAST);

  i2c_send_addr_frame(12, WRITE);
  i2c_send_data_frame(0x13, LAST);

  i2c_send_addr_frame(16, WRITE);
  i2c_send_data_frame(0x22, LAST);

  if (test == 0xA5) print_uart("[UART] Test byte OK \n");
  // extra cycles to stabilize prints
  for (int i=0; i<50; i++) asm("nop");  
  
  return 0;
}
