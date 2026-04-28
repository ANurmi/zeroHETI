/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include <zephyr/drivers/interrupt_controller/riscv_clic.h>
#include <i2c/i2c.h>
#include "board.h"
#include "mailbox.h"
#include "motor.h"

/*Timers OFS*/
#define REP_PERIOD_TICKS  US_TO_TICKS(4000U)
#define REP_OFS0_TICKS    US_TO_TICKS(3900U)
#define REP_OFS1_TICKS    US_TO_TICKS(2900U)
#define REP_OFS2_TICKS    US_TO_TICKS(1900U)
#define REP_OFS3_TICKS    US_TO_TICKS( 900U)

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

/* Shared state */
static volatile uint32_t t_volt[4];
static volatile uint32_t rep3_count;
static volatile uint8_t sim_finished;
static uint64_t sim_start_cycles;

static inline void mbx_report_speed(uint8_t idx, uint32_t speed)
{
    uint64_t t = k_cycle_get_64();
    sys_write32((uint32_t)(t & 0xffffffff), MBX_TIME_LO_ADDR);
    sys_write32((uint32_t)(t >> 32),        MBX_TIME_HI_ADDR);
    sys_write32(speed, mbx_stat_addr[idx]);
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

static inline uint32_t motor_read_speed(uint8_t idx)
{
    uint8_t buf[4] = {0};
    unsigned int key = irq_lock();
    i2c_read_tx(stat_m[idx], buf, 4);
    irq_unlock(key);
    
    //Reconstruct the speed word
    return ((uint32_t)buf[0])       |
           ((uint32_t)buf[1] << 8)  |
           ((uint32_t)buf[2] << 16) |
           ((uint32_t)buf[3] << 24);
}

static inline void motor_write_ctrl(uint8_t idx, uint8_t voltage)
{
    uint8_t buf[2] = {0x00, voltage};

    //Hold the lock for any I2C transaction 
    unsigned int key = irq_lock();
    i2c_write_tx(ctrl_m[idx], buf, 2);
    irq_unlock(key);
}

static inline void motor_write_tune(uint8_t idx, int16_t tune)
{
    uint8_t buf[2] = {
        (uint8_t)(tune & 0xff),
        (uint8_t)(tune >> 8)
    };
    unsigned int key = irq_lock();
    i2c_write_tx(tune_m[idx], buf, 2);
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

	/* Stop simulation */
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
	uint32_t speed = motor_read_speed(0);

	mbx_report_speed(0, speed);
}

static void isr_timer1cmp(void *arg)
{
	ARG_UNUSED(arg);
	uint32_t speed = motor_read_speed(1);

	mbx_report_speed(1, speed);
}

static void isr_timer2cmp(void *arg)
{
	ARG_UNUSED(arg);
	uint32_t speed = motor_read_speed(2);

	mbx_report_speed(2, speed);
}

static void isr_timer3cmp(void *arg)
{
	ARG_UNUSED(arg);
	uint32_t speed = motor_read_speed(3);

	mbx_report_speed(3, speed);

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
    for (int i = 0; i < NUM_MOTORS; i++) {
        motor_write_ctrl(i, bytes[i]);

		//Save current voltages for later tune computation 
        t_volt[i] = (uint32_t)bytes[i] << 8;
    }

	sys_write32(1, MBX_IRQ_ACK_ADDR);
}
/* WRN ISRs */
static void isr_ext0(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	uint32_t speed_now = motor_read_speed(0);

	//We might need 16bit precision but clearly not now!
	int16_t tune = speed_correct(t_volt[0], speed_now);

	motor_write_tune(0, tune);
}
static void isr_ext1(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	uint32_t speed_now = motor_read_speed(1);
	int16_t tune = speed_correct(t_volt[1], speed_now);

	motor_write_tune(1, tune);
}
static void isr_ext2(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	uint32_t speed_now = motor_read_speed(2);
	int16_t tune = speed_correct(t_volt[2], speed_now);

	motor_write_tune(2, tune);
}
static void isr_ext3(void *arg) 
{ 
  	ARG_UNUSED(arg); 
	uint32_t speed_now = motor_read_speed(3);
	int16_t tune = speed_correct(t_volt[3], speed_now);

	motor_write_tune(3, tune);
}

int main(void)
{
	printf("Motor Control demo %s\n", CONFIG_BOARD_TARGET);
	
	i2c_init(I2C_PRESCALER);
	
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