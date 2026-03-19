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

#define CLIC_NODE DT_COMPAT_GET_ANY_STATUS_OKAY(riscv_zeroheti_clic)

#if !DT_NODE_HAS_STATUS(CLIC_NODE, okay)
#error "No okay Zeroheti CLIC node found"
#endif
#define BUSY_WAIT 100
#define IRQN26 26
#define IRQN27 27
#define CLIC_BASE_ADDR  DT_REG_ADDR(CLIC_NODE)
#define CLIC_SLOT(irq)   (CLIC_BASE_ADDR + 0x1000 + ((irq) * 4))
#define CLIC_IP(irq)     (CLIC_SLOT(irq) + 0)
#define CLIC_IE(irq)     (CLIC_SLOT(irq) + 1)
#define CLIC_ATTR(irq)   (CLIC_SLOT(irq) + 2)
#define CLIC_CTL(irq)   (CLIC_SLOT(irq) + 3)

static volatile uint32_t halt = 1;

static volatile uint32_t hits = 0;
static volatile bool isr26_entered = false;
static volatile bool isr27_entered = false;
static volatile bool nesting_worked = false;

void riscv_clic_irq_set_pending(uint32_t irq);
static void clic_isr27(void)
{
    isr27_entered = true;
    hits += 100;
}

static void clic_isr26(void)
{
    isr26_entered = true;
    hits += 1;

    // Re-enable interrupts
    __asm__ volatile("csrsi mstatus, 0x8");
    riscv_clic_irq_set_pending(IRQN27);

    for (volatile int i = 0; i < BUSY_WAIT; i++) {
		;
    }
	
	if (isr27_entered) {
        nesting_worked = true;
    }

    hits += 2;
}
int main(void)
{
    printf("CLIC Demonstrator on %s\n", CONFIG_BOARD_TARGET);
	
    uint32_t halt 	  =1;	

    /** 
     * Populate the interrupt table with the interrupt's parameters.
     * Set the priority in the interrupt controller at runtime.
     * Enable vectoring .shv = 1
    */
    IRQ_CONNECT(IRQN26, 0x01, clic_isr26, NULL, 1);
    IRQ_CONNECT(IRQN27, 0x02, clic_isr27, NULL, 1);
	riscv_clic_irq_vector_set(IRQN26);
    riscv_clic_irq_vector_set(IRQN27);
    irq_enable(IRQN26);
    irq_enable(IRQN27);

     printf("cliccfg=%08x, clicinfo=%08x\n", sys_read32(CLIC_BASE_ADDR), sys_read32((CLIC_BASE_ADDR + 0x0004)));
    //Pend an interrupt 
	riscv_clic_irq_set_pending(IRQN26);
	
    //currently causes hang
	//k_busy_wait(10); 
	for(volatile int  i=0; i < BUSY_WAIT; i++){
		halt+=i;
	}
	
    printf("hits=%u isr26=%d isr27=%d nested=%d\n",
           hits, isr26_entered, isr27_entered, nesting_worked);

    if (nesting_worked) {
        printf("[CLIC] PASS nested interrupt worked\n");
    } else {
        printf("[CLIC] FAIL no nesting\n");
    }
	
  	debug_signal_pass();

    return 0;
}