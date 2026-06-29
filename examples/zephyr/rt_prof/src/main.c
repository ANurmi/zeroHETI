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
#define LOAD_FACTOR       50U
#define RUNTIME_MS        100U

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
}
static void isr_ctrl1(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_ctrl2(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_ctrl3(const void *arg)
{ 
    ARG_UNUSED(arg); 
}

static void isr_rep0(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_rep1(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_rep2(const void *arg)
{ 
    ARG_UNUSED(arg); 
}
static void isr_rep3(const void *arg)
{ 
    ARG_UNUSED(arg); 
}

int main(void)
{
	printf("rt-prof demo %s\n", CONFIG_BOARD_TARGET);

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

    debug_signal_pass();
	return 0;
}