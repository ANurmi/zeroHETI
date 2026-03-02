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
#elif DT_HAS_COMPAT_STATUS_OKAY(nuclei_eclic)
#define CLIC_NODE DT_COMPAT_GET_ANY_STATUS_OKAY(nuclei_eclic)
#else
#error "No okay CLIC node found"
#endif
#define IRQN  26


volatile uint32_t isr_hits;
volatile uint32_t halt;


extern char _irq_vector_table[]; 


void riscv_clic_irq_set_pending(uint32_t irq);
static void clic_isr(const void *arg)
{
    (void)arg;
    isr_hits++;
}

int main(void)
{
    printf("CLIC Demonstrator on %s\n", CONFIG_BOARD_TARGET);
    //csr_write(0x307, 0x00010100);

	csr_write(0x307, (uintptr_t)_irq_vector_table);
	isr_hits = 0;
	halt = 1;

	//csr_write(0x307,0x00010100);

	csr_write(0x347,0x0);

    IRQ_CONNECT(IRQN, 0, clic_isr, NULL, 0);
	//irq_unlock(0);
	// Manually set MIE (bit 3) in mstatus
	//csr_set(0x300, 0x8);
	riscv_clic_irq_vector_set(IRQN);
    riscv_clic_irq_priority_set(IRQN, 1, 0);
	//uint32_t before_enable = sys_read32(0x1069);
    riscv_clic_irq_enable(IRQN);

	//uint32_t after_enable = sys_read32(0x1069);

	//printf("CLIC_CTRL(%d) before_enable=0x%08x after_enable=0x%08x\n",
    //IRQN, before_enable, after_enable);

	printf("enabled? %d\n", riscv_clic_irq_is_enabled(IRQN));

	uint32_t w_before = sys_read32(0x1068);
    riscv_clic_irq_set_pending(IRQN);
	uint32_t w_after = sys_read32(0x1068);

	printf("CLIC_CTRL(%d) before=0x%08x after=0x%08x\n",
       IRQN, w_before, w_after);

	unsigned long mstatus = csr_read(0x300);	
	printf("mstatus = 0x%lx\n", mstatus);

	unsigned long mtvt = csr_read(0x307);	
	printf("mtvt = 0x%lx\n", mtvt);
	
	unsigned long mtvec = csr_read(0x305);	
	printf("mtvec = 0x%lx\n", mtvec);

	unsigned long mnxti = csr_read(0x345);	
	printf("mnxti = 0x%lx\n", mnxti);


	printf("base = 0x%lx\n", (unsigned long)CLIC_CTRL(IRQN));
	printf("INTIP(26) addr = 0x%lx\n", (unsigned long)(CLIC_INTIP(IRQN)));
	printf("INTIE(26) addr = 0x%lx\n", (unsigned long)(CLIC_INTIE(IRQN)));

	printf("CLIC DT base = 0x%lx\n", (unsigned long)DT_REG_ADDR(CLIC_NODE));
	
	k_busy_wait(10);
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