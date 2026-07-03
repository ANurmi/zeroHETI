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
#define LOAD_FACTOR       80U
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
static void finish_sim(void);
static inline void check_runtime(void)
{
    if ((k_cycle_get_64() - sim_start_cycles) >= (uint64_t)RUNTIME_MS * (CPU_FREQ_HZ / 1000U))
        finish_sim();
}

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
    check_runtime();

    /* read inbox directives while not empty*/
    while (!(sys_read32(MBX_STAT) & 0x1)) {         
        uint32_t addr, data;
        read_letter(&addr, &data);
        int motor_index = (addr == 0x100) ? 0 : (addr == 0x101) ? 1 : (addr == 0x102) ? 2 : (addr == 0x103) ? 3 : -1;
        if (motor_index >= 0) {
            period_directive[motor_index] = data;
            send_letter(TASK_ACK(TASK_UPD(motor_index)), 1);
            clic_pend_irq(IRQ_TIMER_OVF(motor_index));
        }
    }

    sys_write32(MBX_CTRL_IRQ_CLR, MBX_CTRL);
    send_letter(TASK_ACK(TASK_MBX), 0);

}

static void isr_upd0(const void *arg)
{
    ARG_UNUSED(arg);

    send_letter(TASK_ACK(TASK_CTRL(0)), 0);
    send_letter(TASK_ACK(TASK_REP(0)),  0);

    //new directive period
    uint32_t nperiod = period_directive[0];
    uint32_t base = TIMER_BASE(0);
    sys_write32(0x0, TIMER_CTRL(base));            
    sys_write32(US_TO_TICKS(nperiod), TIMER_CMP(base));
    sys_write32(0x1, TIMER_CTRL(base));       

    //new control period + deadline
    send_letter(TASK_PERIOD(TASK_CTRL(0)),   nperiod);
    send_letter(TASK_DEADLINE(TASK_CTRL(0)), nperiod);

    //retire UpdateCtrl
    send_letter(TASK_ACK(TASK_UPD(0)), 0);

}

static void isr_upd1(const void *arg)
{
    ARG_UNUSED(arg);

    send_letter(TASK_ACK(TASK_CTRL(1)), 0);
    send_letter(TASK_ACK(TASK_REP(1)),  0);

    //new directive period
    uint32_t nperiod = period_directive[1];
    uint32_t base = TIMER_BASE(1);
    sys_write32(0x0, TIMER_CTRL(base));            
    sys_write32(US_TO_TICKS(nperiod), TIMER_CMP(base));
    sys_write32(0x1, TIMER_CTRL(base));       

    //new control period + deadline
    send_letter(TASK_PERIOD(TASK_CTRL(1)),   nperiod);
    send_letter(TASK_DEADLINE(TASK_CTRL(1)), nperiod);

    //retire UpdateCtrl
    send_letter(TASK_ACK(TASK_UPD(1)), 0);

}
static void isr_upd2(const void *arg)
{
    ARG_UNUSED(arg);

    send_letter(TASK_ACK(TASK_CTRL(2)), 0);
    send_letter(TASK_ACK(TASK_REP(2)),  0);

    //new directive period
    uint32_t nperiod = period_directive[2];
    uint32_t base = TIMER_BASE(2);
    sys_write32(0x0, TIMER_CTRL(base));            
    sys_write32(US_TO_TICKS(nperiod), TIMER_CMP(base));
    sys_write32(0x1, TIMER_CTRL(base));       

    //new control period + deadline
    send_letter(TASK_PERIOD(TASK_CTRL(2)),   nperiod);
    send_letter(TASK_DEADLINE(TASK_CTRL(2)), nperiod);

    //retire UpdateCtrl
    send_letter(TASK_ACK(TASK_UPD(2)), 0);

}
static void isr_upd3(const void *arg)
{
    ARG_UNUSED(arg);

    send_letter(TASK_ACK(TASK_CTRL(3)), 0);
    send_letter(TASK_ACK(TASK_REP(3)),  0);

    //new directive period
    uint32_t nperiod = period_directive[3];
    uint32_t base = TIMER_BASE(3);
    sys_write32(0x0, TIMER_CTRL(base));            
    sys_write32(US_TO_TICKS(nperiod), TIMER_CMP(base));
    sys_write32(0x1, TIMER_CTRL(base));       

    //new control period + deadline
    send_letter(TASK_PERIOD(TASK_CTRL(3)),   nperiod);
    send_letter(TASK_DEADLINE(TASK_CTRL(3)), nperiod);

    //retire UpdateCtrl
    send_letter(TASK_ACK(TASK_UPD(3)), 0);

}

static void isr_ctrl0(const void *arg)
{
    ARG_UNUSED(arg);

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

}

static void isr_ctrl1(const void *arg)
{ 
    ARG_UNUSED(arg);

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

}

static void isr_ctrl2(const void *arg)
{ 
    ARG_UNUSED(arg);

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

}
static void isr_ctrl3(const void *arg)
{ 
    ARG_UNUSED(arg);

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

}

static void isr_rep0(const void *arg)
{
    ARG_UNUSED(arg);
    check_runtime(); 

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)0 << 8) | motor_status[0];
    send_letter(MBX_PRINT_ADDR, rep_letter);          

    send_letter(TASK_ACK(TASK_REP(0)), 0);            

}
static void isr_rep1(const void *arg)
{
    ARG_UNUSED(arg);
    check_runtime(); 

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)1 << 8) | motor_status[1];
    send_letter(MBX_PRINT_ADDR, rep_letter);

    send_letter(TASK_ACK(TASK_REP(1)), 0);

}
static void isr_rep2(const void *arg)
{
    ARG_UNUSED(arg);
    check_runtime();

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)2 << 8) | motor_status[2];
    send_letter(MBX_PRINT_ADDR, rep_letter);          
    send_letter(TASK_ACK(TASK_REP(2)), 0);            

}
static void isr_rep3(const void *arg)
{
    ARG_UNUSED(arg);
    check_runtime();

    uint32_t time_us =
        (uint32_t)((k_cycle_get_64() - sim_start_cycles) / (CPU_FREQ_HZ / 1000000U));

    uint32_t rep_letter = (time_us << 16) | ((uint32_t)3 << 8) | motor_status[3];
    send_letter(MBX_PRINT_ADDR, rep_letter);          

    send_letter(TASK_ACK(TASK_REP(3)), 0); 

}

static void finish_sim(void)
{
    irq_lock();                                  

    uint64_t total_cc = k_cycle_get_64() - sim_start_cycles;

    send_letter(SIM_STOP, 1);                   

    for (volatile int i = 0; i < 100; i++)
        __asm__ volatile("nop");

    uint64_t instret, active_cc;
    READ_CSR64(minstreth, minstret, instret);  
    READ_CSR64(mcycleh,   mcycle,   active_cc); 

    printf("- Retired instructions:      %llu\n", (unsigned long long)instret);
    printf("- Total time w/o setup (cc): %llu\n", (unsigned long long)total_cc);
    printf("  * Active time        (cc): %llu\n", (unsigned long long)active_cc);
    if (total_cc)
        printf("- CPU utilization       (%%): %llu\n",
               (unsigned long long)(active_cc * 100ULL / total_cc));

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

    /* control deadline */
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

    /* release interrupts */
    mintthresh_write(0x00); 
    
    //Suspend main
    k_sleep(K_FOREVER);
    
    return 0;
}