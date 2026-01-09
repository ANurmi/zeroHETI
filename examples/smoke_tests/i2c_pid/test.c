#include "addr_map.h"
#include "mmio.h"
#include "csr.h"
#include "uart.h"
#include "hetic.h"
#include "mtimer.h"
#include "timer_group.h"
#include "i2c.h"

#define WRITE    1
#define LAST     1
#define NOT_LAST 0
#define READ     0

//uint8_t mtime_flag = 0;

__attribute__((aligned(4)))
void isr_mtimer(void){

	// Terminate power control simulation
	// by sending last I2C frame
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x0, LAST);

	// capture minstret and mcycle (low) counters
	uint32_t minstret  = csr_read(CSR_MINSTRET);
	uint32_t mcycle    = csr_read(CSR_MCYCLE);
	uint32_t minstreth = csr_read(CSR_MINSTRETH);
	uint32_t mcycleh   = csr_read(CSR_MCYCLEH);

	// Print out counters
	print_uart("[MTIME_ISR] CSR mcycle:   ");
	print_uart_u32(mcycleh);
	print_uart_u32(mcycle);
	print_uart("\n");  
	print_uart("[MTIME_ISR] CSR minstret: ");
	print_uart_u32(minstreth);
	print_uart_u32(minstret);
	print_uart("\n");  

	// Write to polled debugger memory to end
	write_u32(0x380, 0x80000000);
	while(1);
}

__attribute__((aligned(4)))
void isr_timer_logging(void){
  i2c_send_addr_frame(0, READ);
	for (int i=0;i<10; i++) asm("nop");
	uint8_t rd = i2c_recv_data_frame(LAST);
	print_uart("[LOGGER_ISR] I2C read: ");
	print_uart_u32((uint32_t)rd);
	print_uart("\n");
}

__attribute__((aligned(4)))
void isr_timer_control(void){}

int main() {

  init_uart(0x0, 0x0);
  print_uart("[UART] PID-control demonstrator on i2c\n");

  // i2c setup
  i2c_set_prescaler(4);
  i2c_core_enable();

	// Set CLIC vector table address
	csr_write(CSR_MTVT, VECTORS_ADDR);

  // MTimer to terminate test
  // 1  s in hex @100MHz = 0x05F5_E100
  // 1 ms,               = 0x0001_86A0
  // 1 us,               = 0x0000_0064
  set_mtimer_cmp(0x186A0);
	set_ie(IRQ_IDX_MTIME);
	set_prio(IRQ_IDX_MTIME, 0xFF);
  start_mtimer();
	
	// TG0 logger task setup
	set_ie(IRQ_IDX_TG0_CMP);
	set_prio(IRQ_IDX_TG0_CMP, 0x1);
	timer_group_set_cmp(0, 0x3800);
	timer_group_start(0);

	// TG1 control task setup
	timer_group_set_cmp(1, 0x3000);
	timer_group_start(1);

  // Writing 1 to address 0 activates
  // power control simulation 
  i2c_send_addr_frame(0, WRITE);
  i2c_send_data_frame(0x1, LAST);
	
	// clear instruction & cycle counters
	csr_write(CSR_MINSTRETH, 0x0);
	csr_write(CSR_MCYCLEH,   0x0);
	csr_write(CSR_MINSTRET,  0x0);
	csr_write(CSR_MCYCLE,    0x0);
	
	// Global enable
	interrupts_enable();

  /* START CRITICAL SECTION*/
  //for(int i=0;i<1000;i++) asm("nop");
  //i2c_send_addr_frame(4, WRITE);
  //i2c_send_data_frame(0x22, LAST);
  //while(!get_ip(IRQ_IDX_MTIME));
  /* END CRITICAL SECTION*/

	while(1){
		interrupts_enable();
		asm("wfi");
	}
  
  return 0;
}
