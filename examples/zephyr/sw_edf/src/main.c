/*
 * SPDX-License-Identifier: Apache-2.0
 *
 * de-risk: the thread survives a job and parks again.
 */
#include <stdio.h>
#include <stdint.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include <zephyr/arch/riscv/csr.h>
#include <i2c/i2c.h>
#include "board.h"
#include "motor.h"

//EDFIC bit field
#define INTC_BASE       0x00100000U
#define EDFIC_LINE(n)   (INTC_BASE + 4U * (n))
#define EDFIC_IE        (1U << 0)
#define EDFIC_IP        (1U << 1)
#define TO_DL(prio)     (255U - (uint32_t)(prio))

#define CTRL_PERIOD_US    2000U
#define DEADLINE_CTRL_US  0x1000U

#define THREAD_PRIO 5
#define STACK_SIZE    1024

//Control access to ISRs
static K_SEM_DEFINE(sem, 0, 1);
static K_THREAD_STACK_DEFINE(stack, STACK_SIZE);

static struct k_thread th;
static volatile uint32_t jobs;
static volatile uint32_t trig_time;
static volatile uint32_t wake_time;
static volatile uint32_t latency_cc;

static int32_t  pid_integral[1];
static int16_t  pid_prev_err[1];

static inline void edfic_setup(uint32_t irq, uint32_t prio)
{
    sys_write32((TO_DL(prio) << 8) | EDFIC_IE, EDFIC_LINE(irq));
}
static inline void edfic_pend(uint32_t irq)
{
    sys_write32(sys_read32(EDFIC_LINE(irq)) | EDFIC_IP, EDFIC_LINE(irq));
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

    return (uint8_t)out;
}

static void thread_body(void *a, void *b, void *c)
{
    ARG_UNUSED(a); ARG_UNUSED(b); ARG_UNUSED(c);

    while (1) {
        k_sem_take(&sem, K_FOREVER);
        wake_time = k_cycle_get_32();
        latency_cc = wake_time - trig_time;

        uint8_t measured, out;

        i2c_read_tx(I2C_MOTOR_ADDR(0), &measured, 1);

        out = compute_pid(measured, 0);

        i2c_write_tx(I2C_MOTOR_ADDR(0), &out, 1);

        jobs++;
        printf("  [ctrl] iteration %u done, latency_cc=%u\n", jobs, latency_cc);
        if(jobs == 5) debug_signal_pass();
    }
}
static void isr_ctrl(const void *arg)
{
    __asm__ volatile("csrsi mstatus, 0x8");

    uint32_t now = k_cycle_get_32();
    trig_time = now;
    k_thread_absolute_deadline_set(&th,  now + US_TO_TICKS(DEADLINE_CTRL_US));
    k_sem_give(&sem);
    
    __asm__ volatile("csrci mstatus, 0x8");
}

int main(void)
{
    printf("main prio=%d  thread prio=%d\n",
           k_thread_priority_get(k_current_get()), THREAD_PRIO);
    i2c_init(I2C_PRESCALER);
    
    k_thread_create(&th, stack, STACK_SIZE, thread_body,
                    NULL, NULL, NULL, THREAD_PRIO, 0, K_NO_WAIT);
        
    /* Setup IRQ */
    IRQ_CONNECT(IRQ_TIMER_CMP(0), PRIO_PID, isr_ctrl, NULL, 1);
    edfic_setup(IRQ_TIMER_CMP(0), PRIO_PID);

    // Block all interrupts
    mintthresh_write(0xFF);

    /* config and start timers */
    uint32_t base = TIMER_BASE(0);
    sys_write32(US_TO_TICKS(CTRL_PERIOD_US), TIMER_CMP(base));
    sys_write32(0x1, TIMER_CTRL(base));

    /* drop main below the threads */
    k_thread_priority_set(k_current_get(), K_LOWEST_APPLICATION_THREAD_PRIO);

    printf("after priority drop: ...\n");

    /* release interrupts */
    mintthresh_write(0x00); 
    
    //Suspend main
    k_sleep(K_FOREVER);

    return 0;
}
