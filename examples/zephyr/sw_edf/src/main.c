/*
 * SPDX-License-Identifier: Apache-2.0
 *
 * Software-EDF de-risk. Stage 3: the thread survives a job and parks again.
 */
#include <stdio.h>
#include <stdint.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include <zephyr/arch/riscv/csr.h>
#include "board.h"
#include <zephyr/drivers/interrupt_controller/riscv_clic.h>

#define THREAD_PRIO 5
#define STACK_SIZE    1024

//Control access to ISRs
static K_SEM_DEFINE(sem_long, 0, 1);
static K_SEM_DEFINE(sem_short, 0, 1);
static K_THREAD_STACK_DEFINE(stack_long, STACK_SIZE);
static K_THREAD_STACK_DEFINE(stack_short, STACK_SIZE);

static struct k_thread th_long;
static struct k_thread th_short;
static volatile uint32_t jobs_long;
static volatile uint32_t jobs_short;

static volatile uint32_t thread_trig_time;
static volatile uint32_t thread_wake_time;
static volatile uint32_t measured_latency;

static void body_long(void *a, void *b, void *c)
{
    ARG_UNUSED(a); ARG_UNUSED(b); ARG_UNUSED(c);

    while (1) {
        k_sem_take(&sem_long, K_FOREVER);
        jobs_long++;
        printf("  [th_long] job %u\n", jobs_long);
    }
}

static void body_short(void *a, void *b, void *c)
{
    ARG_UNUSED(a); ARG_UNUSED(b); ARG_UNUSED(c);

    while (1) {
        k_sem_take(&sem_short, K_FOREVER);
        thread_wake_time = k_cycle_get_32();
        measured_latency = thread_wake_time - thread_trig_time;
        jobs_short++;
        printf("  [th_short] job %u, latency_cc=%u\n", jobs_short, measured_latency);
    }
}

int main(void)
{
    printf("main prio=%d  thread prio=%d\n",
           k_thread_priority_get(k_current_get()), THREAD_PRIO);
    
    k_thread_create(&th_long, stack_long, STACK_SIZE, body_long,
                    NULL, NULL, NULL, THREAD_PRIO, 0, K_NO_WAIT);

    k_thread_create(&th_short, stack_short, STACK_SIZE, body_short,
                    NULL, NULL, NULL, THREAD_PRIO, 0, K_NO_WAIT);
    
    uint32_t now = k_cycle_get_32();
    k_thread_absolute_deadline_set(&th_long,  now + 100000);
    k_thread_absolute_deadline_set(&th_short, now + 1000);

    thread_trig_time = k_cycle_get_32();
    k_sem_give(&sem_long);
    k_sem_give(&sem_short);

    /* drop main below the threads */
    k_thread_priority_set(k_current_get(), K_LOWEST_APPLICATION_THREAD_PRIO);

    printf("after priority drop: ...\n");

    debug_signal_pass();
    return 0;
}
