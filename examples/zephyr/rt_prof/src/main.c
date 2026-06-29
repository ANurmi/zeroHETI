/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include <zephyr/drivers/interrupt_controller/riscv_clic.h>
#include <i2c/i2c.h>
#include "board.h"
#include "mailbox.h"
#include "motor.h"

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

#define CTRL_PERIOD_US    2000U
#define DEADLINE_MBX_US   1000U
#define DEADLINE_CTRL_US  1000U
#define DEADLINE_UPD_US   2000U
#define DEADLINE_REP_US   2000U
#define SIM_PRESCALER_VAL 10U
#define RANDOM_SEED       0xB0110c55U
#define LOAD_FACTOR       10U
#define RUNTIME_MS        10U

/* Shared state */
static volatile uint32_t period_directive[4];
static volatile uint8_t  motor_status[4];
static int32_t  pid_integral[4];
static int16_t  pid_prev_err[4];
static uint64_t sim_start_cycles;
static const uint8_t irqs[] = {
    IRQ_TIMER_OVF(0), IRQ_TIMER_OVF(1), IRQ_TIMER_OVF(2), IRQ_TIMER_OVF(3),
    IRQ_TIMER_CMP(0), IRQ_TIMER_CMP(1), IRQ_TIMER_CMP(2), IRQ_TIMER_CMP(3),
    IRQ_MBX,
    IRQ_EXT(0), IRQ_EXT(1), IRQ_EXT(2), IRQ_EXT(3),
};
static struct k_timer finish_timer;

static uint8_t compute_pid(uint8_t input, int idx)
{
    const int16_t SETPOINT = 127;
    const int32_t KP = 1, KI = 1, KD = 1;
    const int32_t INTEGRAL_MAX = 10000;

    int16_t err = SETPOINT - (int16_t)input;

    int32_t integ = pid_integral[idx] + err;         
    if (integ >  INTEGRAL_MAX) integ =  INTEGRAL_MAX;  
    if (integ < -INTEGRAL_MAX) integ = -INTEGRAL_MAX;
    pid_integral[idx] = integ;

    int32_t deriv = (int32_t)(err - pid_prev_err[idx]);
    pid_prev_err[idx] = err;

    int32_t out = SETPOINT + (KP * err + KI * integ + KD * deriv);
    if (out < 0)   out = 0;
    if (out > 255) out = 255;

    motor_status[idx] = input; 
    return (uint8_t)out;
}

static void isr_getmail(const void *arg)
{ 
    ARG_UNUSED(arg); 
}

static void isr_upd0(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_upd1(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_upd2(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_upd3(const void *arg)
{ 
    ARG_UNUSED(arg); 
}

static void isr_ctrl0(const void *arg)
{
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_PID);   

    uint8_t measured;
    unsigned int key = irq_lock();
    i2c_read_tx(I2C_MOTOR_ADDR(0), &measured, 1);      
    irq_unlock(key);

    uint8_t out = compute_pid(measured, 0);

    key = irq_lock();
    i2c_write_tx(I2C_MOTOR_ADDR(0), &out, 1);          
    irq_unlock(key);

    send_letter(TASK_ACK(TASK_CTRL(0)), 0);            
    clic_pend_irq(IRQ_EXT(0));                          
    send_letter(TASK_ACK(TASK_REP(0)), 1);             

    mintthresh_write(prev);                           
}

static void isr_ctrl1(const void *arg)
{ 
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_PID);   

    uint8_t measured;
    unsigned int key = irq_lock();
    i2c_read_tx(I2C_MOTOR_ADDR(1), &measured, 1);     
    irq_unlock(key);

    uint8_t out = compute_pid(measured, 1);

    key = irq_lock();
    i2c_write_tx(I2C_MOTOR_ADDR(1), &out, 1);          
    irq_unlock(key);

    send_letter(TASK_ACK(TASK_CTRL(1)), 0);            
    clic_pend_irq(IRQ_EXT(1));                          
    send_letter(TASK_ACK(TASK_REP(1)), 1);             

    mintthresh_write(prev);                            
}

static void isr_ctrl2(const void *arg)
{ 
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_PID);   

    uint8_t measured;
    unsigned int key = irq_lock();
    i2c_read_tx(I2C_MOTOR_ADDR(2), &measured, 1);      
    irq_unlock(key);

    uint8_t out = compute_pid(measured, 2);

    key = irq_lock();
    i2c_write_tx(I2C_MOTOR_ADDR(2), &out, 1);          
    irq_unlock(key);

    send_letter(TASK_ACK(TASK_CTRL(2)), 0);            
    clic_pend_irq(IRQ_EXT(2));                          
    send_letter(TASK_ACK(TASK_REP(2)), 1);             

    mintthresh_write(prev);                           
}
static void isr_ctrl3(const void *arg)
{ 
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_PID);  

    uint8_t measured;
    unsigned int key = irq_lock();
    i2c_read_tx(I2C_MOTOR_ADDR(3), &measured, 1);      
    irq_unlock(key);

    uint8_t out = compute_pid(measured, 3);

    key = irq_lock();
    i2c_write_tx(I2C_MOTOR_ADDR(3), &out, 1);          
    irq_unlock(key);

    send_letter(TASK_ACK(TASK_CTRL(3)), 0);            
    clic_pend_irq(IRQ_EXT(3));                          
    send_letter(TASK_ACK(TASK_REP(3)), 1);             

    mintthresh_write(prev);                            
}

static void isr_rep0(const void *arg)
{
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_REP);  
    __asm__ volatile("csrsi mstatus, 0x8");           

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)0 << 8) | motor_status[0];
    send_letter(MBX_PRINT_ADDR, rep_letter);          

    send_letter(TASK_ACK(TASK_REP(0)), 0);            

    mintthresh_write(prev);                            
}
static void isr_rep1(const void *arg)
{
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_REP);   
    __asm__ volatile("csrsi mstatus, 0x8");           

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)1 << 8) | motor_status[1];
    send_letter(MBX_PRINT_ADDR, rep_letter);

    send_letter(TASK_ACK(TASK_REP(1)), 0);

    mintthresh_write(prev);                           
}
static void isr_rep2(const void *arg)
{
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_REP);   
    __asm__ volatile("csrsi mstatus, 0x8");           

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)2 << 8) | motor_status[2];
    send_letter(MBX_PRINT_ADDR, rep_letter);          
    send_letter(TASK_ACK(TASK_REP(2)), 0);            

    mintthresh_write(prev);                            
}
static void isr_rep3(const void *arg)
{
    ARG_UNUSED(arg);
    unsigned int prev = mintthresh_write(PRIO_REP);
    __asm__ volatile("csrsi mstatus, 0x8");       

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)3 << 8) | motor_status[3];
    send_letter(MBX_PRINT_ADDR, rep_letter);          

    send_letter(TASK_ACK(TASK_REP(3)), 0); 

    mintthresh_write(prev);
}

static void finish_sim(struct k_timer *timer)
{
    ARG_UNUSED(timer);
    debug_signal_pass();
}
int main(void)
{
	printf("rt-prof demo %s\n", CONFIG_BOARD_TARGET);
	
    i2c_init(I2C_PRESCALER);

    /* Setup IRQs */
    IRQ_CONNECT(IRQ_MBX,          PRIO_MAIL, isr_getmail, NULL, 1);

    IRQ_CONNECT(IRQ_TIMER_OVF(0), PRIO_UPD,  isr_upd0,    NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_OVF(1), PRIO_UPD,  isr_upd1,    NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_OVF(2), PRIO_UPD,  isr_upd2,    NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_OVF(3), PRIO_UPD,  isr_upd3,    NULL, 1);
    
    IRQ_CONNECT(IRQ_TIMER_CMP(0), PRIO_PID,  isr_ctrl0,   NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(1), PRIO_PID,  isr_ctrl1,   NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(2), PRIO_PID,  isr_ctrl2,   NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(3), PRIO_PID,  isr_ctrl3,   NULL, 1);

    IRQ_CONNECT(IRQ_EXT(0),       PRIO_REP,  isr_rep0,    NULL, 1);
    IRQ_CONNECT(IRQ_EXT(1),       PRIO_REP,  isr_rep1,    NULL, 1);
    IRQ_CONNECT(IRQ_EXT(2),       PRIO_REP,  isr_rep2,    NULL, 1);
    IRQ_CONNECT(IRQ_EXT(3),       PRIO_REP,  isr_rep3,    NULL, 1);

    /* vector-set + enable */
    for (size_t i = 0; i < ARRAY_SIZE(irqs); i++) {
        riscv_clic_irq_vector_set(irqs[i]);
        irq_enable(irqs[i]);
    }

    // Block all interrupts
    mintthresh_write(0xFF);

    /* mailbox deadline */
    send_letter(TASK_DEADLINE(TASK_MBX), DEADLINE_MBX_US);

    /* control periods */
    for (size_t i = 0; i < NUM_MOTORS; i++) {
        send_letter(TASK_PERIOD(TASK_CTRL(i)), CTRL_PERIOD_US);
    }

    /* Control deadline */
    for (size_t i = 0; i < NUM_MOTORS; i++) {
        send_letter(TASK_DEADLINE(TASK_CTRL(i)), DEADLINE_CTRL_US);
    }
    
    /* update deadline */
    for (size_t i = 0; i < NUM_MOTORS; i++) {
        send_letter(TASK_DEADLINE(TASK_UPD(i)), DEADLINE_UPD_US);
    }

    /* report deadline */
    for (size_t i = 0; i < NUM_MOTORS; i++) {
        send_letter(TASK_DEADLINE(TASK_REP(i)), DEADLINE_REP_US);
    }

    /* set SIM parameters, start SIM */
    send_letter(SIM_PRESCALER, SIM_PRESCALER_VAL);
    send_letter(SIM_LOADFACTOR, LOAD_FACTOR);
    send_letter(SIM_START, 0);

    /* config and start timers */
    for (size_t i = 0; i < NUM_MOTORS; i++) {
        uint32_t base = TIMER_BASE(i);
        sys_write32(US_TO_TICKS(CTRL_PERIOD_US), TIMER_CMP(base));
        sys_write32(0x1, TIMER_CTRL(base));
    }

    sim_start_cycles = k_cycle_get_64();

    k_timer_init(&finish_timer, finish_sim, NULL);
    k_timer_start(&finish_timer, K_MSEC(RUNTIME_MS), K_NO_WAIT);

    /* release interrupts */
    mintthresh_write(0x00); 
    
    //Suspend main
    k_sleep(K_FOREVER);
    
    return 0;
}