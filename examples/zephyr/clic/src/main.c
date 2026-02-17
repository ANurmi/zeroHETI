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

void riscv_clic_irq_set_pending(uint32_t irq);
volatile uint32_t isr_hits;

static void clic_isr(const void *arg)
{
    (void)arg;
    isr_hits++;
}

#define IRQN  16

int main(void)
{
    printf("CLIC Demonstrator on %s\n", CONFIG_BOARD_TARGET);
    
	isr_hits = 0;
    IRQ_CONNECT(IRQN, 1, clic_isr, NULL, 0);
    riscv_clic_irq_priority_set(IRQN, 1, 0);
    riscv_clic_irq_enable(IRQN);
	printf("enabled? %d\n", riscv_clic_irq_is_enabled(IRQN));
    riscv_clic_irq_set_pending(IRQN);
	printf("pended irq %d\n", IRQN);
	
	unsigned long mstatus;
	__asm__ volatile ("csrr %0, mstatus" : "=r"(mstatus));	
	printf("mstatus = 0x%lx\n", mstatus);


    int64_t elapsed = k_uptime_get();
    while ((k_uptime_get() - elapsed) < 100 && isr_hits == 0) {
		printf("CLIC hits=%u\n", (unsigned)isr_hits);
        k_busy_wait(50);
    }

    if (isr_hits) {
        printf("[CLIC] PASS hits=%u\n", (unsigned)isr_hits);
    } else {
        printf("[CLIC] FAIL no ISR\n");
    }

  	debug_signal_pass();

    return 0;
}