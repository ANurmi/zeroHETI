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
#include <zephyr/device.h>
#include <zephyr/devicetree.h>

#if DT_HAS_COMPAT_STATUS_OKAY(riscv_clic)
#define CLIC_NODE DT_COMPAT_GET_ANY_STATUS_OKAY(riscv_clic)
#else
#error "No okay CLIC node found"
#endif

#define IRQN  26

void riscv_clic_irq_set_pending(uint32_t irq);
static void clic_isr(const void *arg)
{
    volatile uint32_t *hits = (volatile uint32_t *)arg;
    (*hits)++;
}

int main(void)
{
    printf("CLIC Demonstrator on %s\n", CONFIG_BOARD_TARGET);
	
	static volatile uint32_t isr_hits = 0;
	uint32_t halt 	  =1;	
	uintptr_t base = DT_REG_ADDR(CLIC_NODE);
	uintptr_t ip   = base + 0x1000 + IRQN*4;
	uintptr_t ie   = base + 0x1001 + IRQN*4;
	uintptr_t attr = base + 0x1002 + IRQN*4;
	uintptr_t ctl  = base + 0x1003 + IRQN*4;

	csr_write(0x347,0x0);
    IRQ_CONNECT(IRQN, 10, clic_isr, &isr_hits, 0);
	
	riscv_clic_irq_priority_set(IRQN, 10, 1);
	riscv_clic_irq_vector_set(IRQN);
    riscv_clic_irq_enable(IRQN);
	
	printf("enabled? %d\n", riscv_clic_irq_is_enabled(IRQN));

	printf("ip=%02x ie=%02x attr=%02x ctl=%02x\n",
			sys_read8(ip), sys_read8(ie), sys_read8(attr), sys_read8(ctl));

	riscv_clic_irq_set_pending(IRQN);

	printf("after set_pending: ip=%02x ie=%02x attr=%02x ctl=%02x\n",
	       sys_read8(ip), sys_read8(ie), sys_read8(attr), sys_read8(ctl));
	
	
	//k_busy_wait(10);
	for(int i=0; i < 200; i++){
		halt+=i;
	}

    if (isr_hits) {
        printf("[CLIC] PASS hits=%u\n", (unsigned)isr_hits);
    } else {
        printf("[CLIC] FAIL no ISR\n");
    }
  	debug_signal_pass();

    return 0;
}