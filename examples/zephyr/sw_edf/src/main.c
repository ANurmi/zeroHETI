/*
 * SPDX-License-Identifier: Apache-2.0
 *
 * Heavy work: scale to 4motors.
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
#include "mailbox.h"

//EDFIC bit field
#define INTC_BASE       0x00100000U
#define EDFIC_LINE(n)   (INTC_BASE + 4U * (n))
#define EDFIC_IE        (1U << 0)
#define EDFIC_IP        (1U << 1)
#define TO_DL(prio)     (255U - (uint32_t)(prio))
#define M(n)            ((void *)(uintptr_t)(n))

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
#define DEADLINE_CTRL_US  4000U
#define DEADLINE_MBX_US   4000U
#define SIM_PRESCALER_VAL 10U
#define LOAD_FACTOR       80U

#define THREAD_PRIO 5
#define STACK_SIZE    1024

//Control access to ISRs
static struct k_sem sem[NUM_MOTORS];
static K_THREAD_STACK_ARRAY_DEFINE(stacks, NUM_MOTORS, STACK_SIZE);

static struct k_thread th[NUM_MOTORS];
static volatile uint32_t jobs;
static uint64_t sim_start_cycles;
static volatile uint32_t trig_time[NUM_MOTORS];
static volatile uint32_t wake_time[NUM_MOTORS];
static volatile uint32_t latency_cc[NUM_MOTORS];

static int32_t  pid_integral[NUM_MOTORS];
static int16_t  pid_prev_err[NUM_MOTORS];

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

static void thread_body(void *a, void *b, void *c)
{
    ARG_UNUSED(b); ARG_UNUSED(c);
    const int n = (int)(uintptr_t)a;

    while (1) {
        k_sem_take(&sem[n], K_FOREVER);
        wake_time[n] = k_cycle_get_32();
        latency_cc[n] = wake_time[n] - trig_time[n];

        uint8_t measured, out;

        i2c_read_tx(I2C_MOTOR_ADDR(n), &measured, 1);

        out = compute_pid(measured, n);

        i2c_write_tx(I2C_MOTOR_ADDR(n), &out, 1);

        send_letter(TASK_ACK(TASK_CTRL(n)), 0);

        jobs++;
        send_letter(MBX_PRINT_ADDR,
                    ((TICKS_TO_US(latency_cc[n]) & 0xFFFFU) << 16) | ((uint32_t)n << 8) | (jobs & 0xFFU));
        if(jobs == NUM_MOTORS * 2) finish_sim();
    }
}
static void isr_getmail(const void *arg)
{
    ARG_UNUSED(arg);

    while (!(sys_read32(MBX_STAT) & 0x1)) {
        uint32_t addr, data;
        read_letter(&addr, &data);
    }

    sys_write32(MBX_CTRL_IRQ_CLR, MBX_CTRL);
    send_letter(TASK_ACK(TASK_MBX), 0);
}

static void isr_ctrl(const void *arg)
{
    __asm__ volatile("csrsi mstatus, 0x8");

    const int n = (int)(uintptr_t)arg;
    uint32_t now = k_cycle_get_32();
    trig_time[n] = now;
    k_thread_absolute_deadline_set(&th[n],  now + US_TO_TICKS(DEADLINE_CTRL_US));
    k_sem_give(&sem[n]);

    __asm__ volatile("csrci mstatus, 0x8");
}

int main(void)
{
    printf("main prio=%d  thread prio=%d\n",
           k_thread_priority_get(k_current_get()), THREAD_PRIO);
    i2c_init(I2C_PRESCALER);

    for (int i = 0; i < NUM_MOTORS; i++) {
        k_sem_init(&sem[i], 0, 1);
        k_thread_create(&th[i], stacks[i], STACK_SIZE, thread_body,
                         M(i), NULL, NULL, THREAD_PRIO, 0, K_NO_WAIT);
    }

    // Setup IRQ
    IRQ_CONNECT(IRQ_MBX,          PRIO_MAIL, isr_getmail, NULL, 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(0), PRIO_PID,  isr_ctrl, M(0), 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(1), PRIO_PID,  isr_ctrl, M(1), 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(2), PRIO_PID,  isr_ctrl, M(2), 1);
    IRQ_CONNECT(IRQ_TIMER_CMP(3), PRIO_PID,  isr_ctrl, M(3), 1);

    edfic_setup(IRQ_MBX, PRIO_MAIL);
    for (int i = 0; i < NUM_MOTORS; i++) {
        edfic_setup(IRQ_TIMER_CMP(i), PRIO_PID);
    }

    // Block all interrupts
    mintthresh_write(0xFF);

    send_letter(TASK_DEADLINE(TASK_MBX), DEADLINE_MBX_US);

    for (int i = 0; i < NUM_MOTORS; i++) {
        send_letter(TASK_PERIOD(TASK_CTRL(i)),   CTRL_PERIOD_US);
        send_letter(TASK_DEADLINE(TASK_CTRL(i)), DEADLINE_CTRL_US);
    }

    send_letter(SIM_PRESCALER,  SIM_PRESCALER_VAL);
    send_letter(SIM_LOADFACTOR, LOAD_FACTOR);
    send_letter(SIM_START, 0);

    /* config and start timers */
    for (int i = 0; i < NUM_MOTORS; i++) {
        uint32_t base = TIMER_BASE(i);
        sys_write32(US_TO_TICKS(CTRL_PERIOD_US), TIMER_CMP(base));
        sys_write32(0x1, TIMER_CTRL(base));
    }

    /* drop main below the threads */
    k_thread_priority_set(k_current_get(), K_LOWEST_APPLICATION_THREAD_PRIO);

    printf("after priority drop: ...\n");

    __asm__ volatile("csrw mcycle,    0");
    __asm__ volatile("csrw mcycleh,   0");
    __asm__ volatile("csrw minstret,  0");
    __asm__ volatile("csrw minstreth, 0");

    sim_start_cycles = k_cycle_get_64();

    /* release interrupts */
    mintthresh_write(0x00);

    //Suspend main
    k_sleep(K_FOREVER);

    return 0;
}
