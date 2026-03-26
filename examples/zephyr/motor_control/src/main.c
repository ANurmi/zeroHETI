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
#include <i2c/i2c.h>

#define PRESCALER 4
#define BUF_BYTES 4

uint8_t tx_buf[BUF_BYTES] = {0};
uint8_t rx_buf[BUF_BYTES] = {0};

/*
 * all_irqs[23:16] = apb_timer_irqs  
 * Select only CMP lines.
 */
#define IRQ_TIMER0CMP  17
#define IRQ_TIMER1CMP  19
#define IRQ_TIMER2CMP  21
#define IRQ_TIMER3CMP  23
#define IRQ_MBX        26
#define IRQ_EXT0       27
#define IRQ_EXT1       28
#define IRQ_EXT2       29
#define IRQ_EXT3       30

/* Interrupt priorities */
#define PRIO_TIMER_CMP 0x88
#define PRIO_EXT       0x10
#define PRIO_MBX       0x03

/* Mailbox addresses */
#define MBX_INBOX_ADDR   0x30000
#define MBX_IRQ_ACK_ADDR 0x30004
#define MBX_TIME_LO_ADDR 0x30008
#define MBX_TIME_HI_ADDR 0x3000C
#define MBX_M0_STAT_ADDR 0x30010
#define MBX_M1_STAT_ADDR 0x30014
#define MBX_M2_STAT_ADDR 0x30018
#define MBX_M3_STAT_ADDR 0x3001C

/* I2C addresses */
#define I2C_SIM_CTRL_ADDR 0
#define I2C_M0_STAT_ADDR  1
#define I2C_M0_CTRL_ADDR  2
#define I2C_M0_TUNE_ADDR  3
#define I2C_M1_STAT_ADDR  4
#define I2C_M1_CTRL_ADDR  5
#define I2C_M1_TUNE_ADDR  6
#define I2C_M2_STAT_ADDR  7
#define I2C_M2_CTRL_ADDR  8
#define I2C_M2_TUNE_ADDR  9
#define I2C_M3_STAT_ADDR  10
#define I2C_M3_CTRL_ADDR  11
#define I2C_M3_TUNE_ADDR  12

/* Fires when simulation time is up */
static void ktimer_elapsed(struct k_timer *timer)
{
	ARG_UNUSED(timer);
	printf("[ktimer] 20ms elapsed\n");
	debug_signal_pass();
}
K_TIMER_DEFINE(sim_ktimer, ktimer_elapsed, NULL);

static void isr_timer0cmp(void *arg)
{
	ARG_UNUSED(arg);
	printf("[ISR] Timer0Cmp\n");
}

static void isr_timer1cmp(void *arg)
{
	ARG_UNUSED(arg);
	printf("[ISR] Timer1Cmp\n");
}

static void isr_timer2cmp(void *arg)
{
	ARG_UNUSED(arg);
	printf("[ISR] Timer2Cmp\n");
}

static void isr_timer3cmp(void *arg)
{
	ARG_UNUSED(arg);
	printf("[ISR] Timer3Cmp\n");
}

static void isr_mbx(void *arg)
{
	ARG_UNUSED(arg);
	printf("[ISR] Mbx\n");
}

static void isr_ext0(void *arg) 
{ 
  ARG_UNUSED(arg); 
  printf("[ISR] Motor0 \n"); 
}
static void isr_ext1(void *arg) 
{ 
  ARG_UNUSED(arg); 
  printf("[ISR] Motor1 \n"); 
}
static void isr_ext2(void *arg) 
{ 
  ARG_UNUSED(arg); 
  printf("[ISR] Motor2 \n"); 
}
static void isr_ext3(void *arg) 
{ 
  ARG_UNUSED(arg); 
  printf("[ISR] Motor3 \n"); 
}


int main(void)
{
	printf("Motor Control demo %s\n", CONFIG_BOARD_TARGET);

	/* I2C init */
	i2c_init(PRESCALER);

	/* Setup IRQS */
	IRQ_CONNECT(IRQ_TIMER0CMP, PRIO_TIMER_CMP, isr_timer0cmp, NULL, 1);
	IRQ_CONNECT(IRQ_TIMER1CMP, PRIO_TIMER_CMP, isr_timer1cmp, NULL, 1);
	IRQ_CONNECT(IRQ_TIMER2CMP, PRIO_TIMER_CMP, isr_timer2cmp, NULL, 1);
	IRQ_CONNECT(IRQ_TIMER3CMP, PRIO_TIMER_CMP, isr_timer3cmp, NULL, 1);
	IRQ_CONNECT(IRQ_MBX,       PRIO_MBX,       isr_mbx,       NULL, 1);
	IRQ_CONNECT(IRQ_EXT0,      PRIO_EXT,       isr_ext0,      NULL, 1);
	IRQ_CONNECT(IRQ_EXT1,      PRIO_EXT,       isr_ext1,      NULL, 1);
	IRQ_CONNECT(IRQ_EXT2,      PRIO_EXT,       isr_ext2,      NULL, 1);
	IRQ_CONNECT(IRQ_EXT3,      PRIO_EXT,       isr_ext3,      NULL, 1);

	riscv_clic_irq_vector_set(IRQ_TIMER0CMP);
	riscv_clic_irq_vector_set(IRQ_TIMER1CMP);
	riscv_clic_irq_vector_set(IRQ_TIMER2CMP);
	riscv_clic_irq_vector_set(IRQ_TIMER3CMP);
	riscv_clic_irq_vector_set(IRQ_MBX);
	riscv_clic_irq_vector_set(IRQ_EXT0);
	riscv_clic_irq_vector_set(IRQ_EXT1);
	riscv_clic_irq_vector_set(IRQ_EXT2);
	riscv_clic_irq_vector_set(IRQ_EXT3);

	irq_enable(IRQ_TIMER0CMP);
	irq_enable(IRQ_TIMER1CMP);
	irq_enable(IRQ_TIMER2CMP);
	irq_enable(IRQ_TIMER3CMP);
	irq_enable(IRQ_MBX);
	irq_enable(IRQ_EXT0);
	irq_enable(IRQ_EXT1);
	irq_enable(IRQ_EXT2);
	irq_enable(IRQ_EXT3);

	/* Start 20ms watchdog */
	k_timer_start(&sim_ktimer, K_MSEC(20), K_NO_WAIT);

	// Start simulation
	tx_buf[0] = 0x1;
	i2c_write_tx(I2C_SIM_CTRL_ADDR, tx_buf, BUF_BYTES);

	/* Sleep */
	k_sleep(K_FOREVER);

	return 0;
}