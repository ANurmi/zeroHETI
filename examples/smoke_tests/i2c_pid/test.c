#include "addr_map.h"
#include "mmio.h"
#include "csr.h"
#include "uart.h"
#include "hetic.h"
#include "mtimer.h"
#include "i2c.h"

#define WRITE    1
#define LAST     1
#define NOT_LAST 0
#define READ     0

//uint8_t mtime_flag = 0;

__attribute__((aligned(4)))
void isr_mtimer(void){
	clear_ie(IRQ_IDX_MTIME);
	print_uart("[MTIME_IRQ] Triggered\n");
	// Terminate test
	write_u32(0x380, 0x80000000);
	asm("wfi");
}
void isr_timer_logging(void){}
void isr_timer_control(void){}

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] PID-control demonstrator on i2c\n");
	//print_uart_u32(0xDEADBEEF);  

  /* GLOBAL SETUP */
  i2c_set_prescaler(4);
  i2c_core_enable();

	csr_write(CSR_MTVT, VECTORS_ADDR);

  // MTimer to terminate test

  // 1  s in hex @100MHz = 0x05F5_E100
  // 1 ms,               = 0x0001_86A0
  // 1 us,               = 0x0000_0064
  set_mtimer_cmp(0x186A0);
	interrupts_enable();
	set_ie(IRQ_IDX_MTIME);
  start_mtimer();

  // Writing 1 to address 0 activates
  // power control simulation 
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x1, LAST);
  /* START CRITICAL SECTION*/
  for(int i=0;i<1000;i++) asm("nop");
  //i2c_send_addr_frame(4, WRITE);
  //i2c_send_data_frame(0x22, LAST);
  //while(!get_ip(IRQ_IDX_MTIME));
  /* END CRITICAL SECTION*/
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x0, LAST);

	asm("wfi");
	interrupts_disable();
  
  return 0;
}
