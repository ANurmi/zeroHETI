/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <debug/debug.h>
#include <zephyr/drivers/interrupt_controller/riscv_clic.h>
#include </home/abdelkadir/workspace_dir/zephyr/drivers/interrupt_controller/intc_clic.h>

#if DT_HAS_COMPAT_STATUS_OKAY(riscv_clic)
#define CLIC_NODE DT_COMPAT_GET_ANY_STATUS_OKAY(riscv_clic)
#else
#error "No okay CLIC node found"
#endif

#define IRQN  26
#define CLIC_BASE_ADDR  DT_REG_ADDR(CLIC_NODE)
#define CLIC_IP(IRQN)   (CLIC_BASE_ADDR + 0x1000 + (IRQN) * 4)
#define CLIC_IE(IRQN)   (CLIC_BASE_ADDR + 0x1001 + (IRQN) * 4)
#define CLIC_ATTR(IRQN) (CLIC_BASE_ADDR + 0x1002 + (IRQN) * 4)
#define CLIC_CTL(IRQN)  (CLIC_BASE_ADDR + 0x1003 + (IRQN) * 4)

static volatile bool interrupt_fired = false;

void riscv_clic_irq_set_pending(uint32_t irq);
static void clic_isr(void)
{
    interrupt_fired = true;
}

int main(void)
{
    printf("CLIC Demonstrator on %s\n", CONFIG_BOARD_TARGET);
	
	uint32_t halt 	  =1;	

    IRQ_CONNECT(IRQN, 0xFF, clic_isr, NULL, 0);
	riscv_clic_irq_priority_set(IRQN, 0xFF, 1);
	riscv_clic_irq_vector_set(IRQN);

	//Force place the interrupt handler address into the mtvt table
	uint32_t *irq_vector_table = (uint32_t *)(0x10100);
	irq_vector_table[IRQN] = (uint32_t)clic_isr;

    riscv_clic_irq_enable(IRQN);

	printf("ip=%02x ie=%02x attr=%02x ctl=%02x\n",
		sys_read8(CLIC_IP(IRQN)), sys_read8(CLIC_IE(IRQN)), 
		sys_read8(CLIC_ATTR(IRQN)), sys_read8(CLIC_CTRL(IRQN)));

	riscv_clic_irq_set_pending(IRQN);

	printf("set_pending: ip=%02x ie=%02x attr=%02x ctl=%02x\n",
	       sys_read8(CLIC_IP(IRQN)), sys_read8(CLIC_IE(IRQN)), 
		   sys_read8(CLIC_ATTR(IRQN)), sys_read8(CLIC_CTRL(IRQN)));

	//currently causes hang
	//k_busy_wait(10); 
	for(volatile int  i=0; i < 200; i++){
		halt+=i;
	}

    if (interrupt_fired) {
        printf("[CLIC] PASS\n");
    } else {
        printf("[CLIC] FAIL no ISR\n");
    }
	
  	debug_signal_pass();

    return 0;
}