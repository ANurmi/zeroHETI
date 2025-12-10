#include "addr_map.h"
#include "mmio.h"
#include "uart.h"
#include "mtimer.h"
#include "hetic.h"

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] zeroHETI i2c test\n");

  write_u32(I2C_CLK_PRESCALER, 4u); // set prescaler to non-zero
  write_u32(I2C_CTRL, (3u << 6)); // set s_core_en, s_ien
  write_u32(I2C_TX,   67u); // set TX buffer to 67

                //  STA   STO       RD      WR      ACK     IA
  uint32_t cmd = (1u<<7 | 0u<<6  | 0u<<5 | 1u<<4 | 0u<<3 | 0u<<0);
  write_u32(I2C_CMD, cmd);

  for (int i=0; i<5000; i++) asm("nop");  
  
  // extra cycles to stabilize prints
  for (int i=0; i<50; i++) asm("nop");  
  return 0;
}
