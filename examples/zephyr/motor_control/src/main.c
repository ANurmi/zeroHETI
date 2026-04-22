/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include <zephyr/drivers/interrupt_controller/riscv_clic.h>
#include <i2c/i2c.h>

/* IRQ lines */
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

/* APB timer registers */
#define TIMER0_BASE  0x3300
#define TIMER1_BASE  0x3310
#define TIMER2_BASE  0x3320
#define TIMER3_BASE  0x3330

#define TIMER_CNT(base)   ((base) + 0x0)
#define TIMER_CTRL(base)  ((base) + 0x4)
#define TIMER_CMP(base)   ((base) + 0x8)

/* Time to cpu ticks */
#define US_TO_TICKS(us)  ((us) * 10U)

#define REP_PERIOD_TICKS  US_TO_TICKS(4000U)
#define REP_OFS0_TICKS    US_TO_TICKS(3900U)
#define REP_OFS1_TICKS    US_TO_TICKS(2900U)
#define REP_OFS2_TICKS    US_TO_TICKS(1900U)
#define REP_OFS3_TICKS    US_TO_TICKS( 900U)

/* I2C slave addresses */
#define I2C_SIM_CTRL_ADDR  0
#define I2C_M0_STAT_ADDR   1
#define I2C_M0_CTRL_ADDR   2
#define I2C_M0_TUNE_ADDR   3
#define I2C_M1_STAT_ADDR   4
#define I2C_M1_CTRL_ADDR   5
#define I2C_M1_TUNE_ADDR   6
#define I2C_M2_STAT_ADDR   7
#define I2C_M2_CTRL_ADDR   8
#define I2C_M2_TUNE_ADDR   9
#define I2C_M3_STAT_ADDR   10
#define I2C_M3_CTRL_ADDR   11
#define I2C_M3_TUNE_ADDR   12

#define PRESCALER 4
#define R_MOTOR 10000
#define READ_CSR64(hi_csr, lo_csr, out)                          \
    do {                                                          \
        uint32_t _lo, _hi, _hi2;                                 \
        do {                                                      \
            __asm__ volatile("csrr %0, " #hi_csr : "=r"(_hi));  \
            __asm__ volatile("csrr %0, " #lo_csr : "=r"(_lo));  \
            __asm__ volatile("csrr %0, " #hi_csr : "=r"(_hi2)); \
        } while (_hi != _hi2);                                   \
        (out) = ((uint64_t)_hi << 32) | _lo;                    \
    } while (0)

static volatile uint32_t t_volt[4];
static volatile uint32_t rep3_count;
static volatile uint8_t sim_finished;
static uint64_t sim_start_cycles;

static inline void mbx_wr_stat(uint32_t stat_addr, uint32_t stat)
{
	uint64_t time_now = k_cycle_get_64();

	sys_write32((uint32_t)(time_now & 0xffffffffULL), MBX_TIME_LO_ADDR);
	sys_write32((uint32_t)(time_now >> 32), MBX_TIME_HI_ADDR);
	sys_write32(stat, stat_addr);
}
static inline uint32_t newton_sqrt(uint32_t val)
{
	if (val < 2U)
		return val;

	uint32_t x = val; //or maybe "val >> 1" if val is large
    for (int i = 0; i < 4; i++) {
        x = (x + val / x) >> 1; 
    }
    return x;
}
static inline int16_t speed_correct(uint32_t voltage_setpoint, uint32_t speed_actual)
{
	uint32_t t_pow = (voltage_setpoint * voltage_setpoint) / (uint32_t)R_MOTOR;
	int32_t delta = (int32_t)t_pow - (int32_t)speed_actual;
	int32_t error_scaled = delta / R_MOTOR;
	int16_t correction = (int16_t)newton_sqrt(abs(error_scaled));

	if (delta < 0) {
		correction = (int16_t)(-correction);
	}

	return correction;
}
static inline uint32_t motor_read_stat(uint8_t stat_addr)
{
	uint8_t buf[4] = {0};
	i2c_read_tx(stat_addr, buf, 4);
	
	//reconstruct the speed word
	return ((uint32_t)buf[0]) |
	       ((uint32_t)buf[1] << 8) |
	       ((uint32_t)buf[2] << 16) |
	       ((uint32_t)buf[3] << 24);
}
static inline void motor_write_ctrl(uint8_t ctrl_addr, uint8_t cmd_hi)
{
	uint8_t buf[2] = {0x00, cmd_hi};
	i2c_write_tx(ctrl_addr, buf, 2);
}
static inline void motor_wr_tune(uint8_t tune_addr, int16_t tune)
{
	unsigned int key = irq_lock();
		uint8_t buf[2] = {
            (uint8_t)(tune & 0xff),
            (uint8_t)(tune >> 8),
	};
	i2c_write_tx(tune_addr, buf, 2);
	irq_unlock(key);
}
static void finish_sim(void)
{
	unsigned int key = irq_lock();

	//Guard against multiple calls 
	if (sim_finished) {
		irq_unlock(key);
		return;
	}
	sim_finished = 1;

	/* Stop periodic timer sources */
	sys_write32(0x0, TIMER_CTRL(TIMER0_BASE));
	sys_write32(0x0, TIMER_CTRL(TIMER1_BASE));
	sys_write32(0x0, TIMER_CTRL(TIMER2_BASE));
	sys_write32(0x0, TIMER_CTRL(TIMER3_BASE));

	/* Stop the simulation */
	uint8_t sim_off = 0;
	i2c_write_tx(I2C_SIM_CTRL_ADDR, &sim_off, 1);
	
	uint64_t instret = 0;
	uint64_t active_cc = 0;
	
	READ_CSR64(minstreth, minstret, instret);
	READ_CSR64(mcycleh,   mcycle,   active_cc);

	uint64_t total_cc = k_cycle_get_64() - sim_start_cycles;

	printf("Instructions retired: %llu, cycles: %llu\n",
	       (unsigned long long)instret,
	       (unsigned long long)active_cc);
	printf("Total time (cc): %llu, active time (cc): %llu,\n",
	       (unsigned long long)total_cc,
	       (unsigned long long)active_cc);
	if (total_cc != 0ULL) {
		printf("CPU utilization: %llu%%\n",
		       (unsigned long long)((active_cc * 100ULL) / total_cc));
	}
	
	debug_signal_pass();
	irq_unlock(key);
}

static void isr_timer0cmp(void *arg)
{
	ARG_UNUSED(arg);
	unsigned int key = irq_lock();
	uint32_t stat = motor_read_stat(I2C_M0_STAT_ADDR);
	irq_unlock(key);

	mbx_wr_stat(MBX_M0_STAT_ADDR, stat);
}

static void isr_timer1cmp(void *arg)
{
	ARG_UNUSED(arg);
	unsigned int key = irq_lock();
	uint32_t stat = motor_read_stat(I2C_M1_STAT_ADDR);
	irq_unlock(key);

	mbx_wr_stat(MBX_M1_STAT_ADDR, stat);
}

static void isr_timer2cmp(void *arg)
{
	ARG_UNUSED(arg);
	unsigned int key = irq_lock();
	uint32_t stat = motor_read_stat(I2C_M2_STAT_ADDR);
	irq_unlock(key);

	mbx_wr_stat(MBX_M2_STAT_ADDR, stat);
}

static void isr_timer3cmp(void *arg)
{
	ARG_UNUSED(arg);
	unsigned int key = irq_lock();
	uint32_t stat = motor_read_stat(I2C_M3_STAT_ADDR);
	irq_unlock(key);

	mbx_wr_stat(MBX_M3_STAT_ADDR, stat);

	//Fifth REP of M3 signals sim termination  
	rep3_count++;
	if (rep3_count >= 5U) {
		finish_sim();
	}
}
/* MBX ISR */
static void isr_mbx(void *arg)
{
	ARG_UNUSED(arg);

	uint32_t mail = sys_read32(MBX_INBOX_ADDR);
	uint8_t bytes[4] = {
		(uint8_t)(mail >> 24),
		(uint8_t)(mail >> 16),
		(uint8_t)(mail >> 8),
		(uint8_t)(mail >> 0),
	};

	//All I2C transactions should be locked
	unsigned int key = irq_lock();
	motor_write_ctrl(I2C_M0_CTRL_ADDR, bytes[0]);
	motor_write_ctrl(I2C_M1_CTRL_ADDR, bytes[1]);
	motor_write_ctrl(I2C_M2_CTRL_ADDR, bytes[2]);
	motor_write_ctrl(I2C_M3_CTRL_ADDR, bytes[3]);
	irq_unlock(key);

	//Save current voltages for later tune computation 
	t_volt[0] = ((uint32_t)bytes[0]) << 8;
	t_volt[1] = ((uint32_t)bytes[1]) << 8;
	t_volt[2] = ((uint32_t)bytes[2]) << 8;
	t_volt[3] = ((uint32_t)bytes[3]) << 8;

	sys_write32(1, MBX_IRQ_ACK_ADDR);
}
/* WRN ISRs */
static void isr_ext0(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	unsigned int key = irq_lock();
	uint32_t speed_now = motor_read_stat(I2C_M0_STAT_ADDR);
	irq_unlock(key);
	//We might need 16bit precision but clearly not now!
	int16_t tune = speed_correct(t_volt[0], speed_now);

	motor_wr_tune(I2C_M0_TUNE_ADDR, tune);
}
static void isr_ext1(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	unsigned int key = irq_lock();
	uint32_t speed_now = motor_read_stat(I2C_M1_STAT_ADDR);
	irq_unlock(key);
	int16_t tune = speed_correct(t_volt[1], speed_now);

	motor_wr_tune(I2C_M1_TUNE_ADDR, tune);
}
static void isr_ext2(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	unsigned int key = irq_lock();
	uint32_t speed_now = motor_read_stat(I2C_M2_STAT_ADDR);
	irq_unlock(key);
	int16_t tune = speed_correct(t_volt[2], speed_now);

	motor_wr_tune(I2C_M2_TUNE_ADDR, tune);
}
static void isr_ext3(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	unsigned int key = irq_lock();
	uint32_t speed_now = motor_read_stat(I2C_M3_STAT_ADDR);
	irq_unlock(key);
	int16_t tune = speed_correct(t_volt[3], speed_now);

	motor_wr_tune(I2C_M3_TUNE_ADDR, tune);
}

int main(void)
{
	printf("Motor Control demo %s\n", CONFIG_BOARD_TARGET);
	
	i2c_init(PRESCALER);
	
	/* Setup IRQs */
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

	// Block all interrupts
	__asm__ volatile ("csrw 0x347, %0" :: "r"(0xFF));

	/*Set timer compare and counter values with 1000us ofs*/
	sys_write32(REP_PERIOD_TICKS, TIMER_CMP(TIMER0_BASE));
	sys_write32(REP_OFS0_TICKS,   TIMER_CNT(TIMER0_BASE));
	sys_write32(REP_PERIOD_TICKS, TIMER_CMP(TIMER1_BASE));
	sys_write32(REP_OFS1_TICKS,   TIMER_CNT(TIMER1_BASE));
	sys_write32(REP_PERIOD_TICKS, TIMER_CMP(TIMER2_BASE));
	sys_write32(REP_OFS2_TICKS,   TIMER_CNT(TIMER2_BASE));
	sys_write32(REP_PERIOD_TICKS, TIMER_CMP(TIMER3_BASE));
	sys_write32(REP_OFS3_TICKS,   TIMER_CNT(TIMER3_BASE));

	k_busy_wait(100);

	//Clear performance counters
	__asm__ volatile("csrwi minstret, 0");
	__asm__ volatile("csrwi mcycle, 0");
	__asm__ volatile("csrwi minstreth, 0");
	__asm__ volatile("csrwi mcycleh, 0");

	//Start all timers simultaneously
	sys_write32(0x1, TIMER_CTRL(TIMER0_BASE));
	sys_write32(0x1, TIMER_CTRL(TIMER1_BASE));
	sys_write32(0x1, TIMER_CTRL(TIMER2_BASE));
	sys_write32(0x1, TIMER_CTRL(TIMER3_BASE));

	//Latch time
	sim_start_cycles = k_cycle_get_64();

	//Kickstart the simulation
	uint8_t sim_on = 1;
	i2c_write_tx(I2C_SIM_CTRL_ADDR, &sim_on, 1);

	// Release interrupts
	__asm__ volatile ("csrw 0x347, %0" :: "r"(0x00));
	
	//Suspend main
	k_sleep(K_FOREVER);

	return 0;
}