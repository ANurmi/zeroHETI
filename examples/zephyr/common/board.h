#ifndef BOARD_H
#define BOARD_H

#include <stdint.h>

// Timing 
#define CPU_FREQ_HZ     10000000U
#define US_TO_TICKS(us) ((us) * (CPU_FREQ_HZ / 1000000U))
#define TICKS_TO_US(ticks) ((ticks) / (CPU_FREQ_HZ / 1000000U))

// I2C
#define I2C_PRESCALER   4

// APB Timers 
#define TIMER_BASE(i) (0x3300 + (i) * 0x10)
#define TIMER_CNT(base)  ((base) + 0x0)
#define TIMER_CTRL(base) ((base) + 0x4)
#define TIMER_CMP(base)  ((base) + 0x8)

// IRQ numbers
#define IRQ_MBX          26
#define IRQ_TIMER_OVF(i) (16 + (i) * 2)
#define IRQ_TIMER_CMP(i) (17 + (i) * 2)
#define IRQ_EXT(i)       (27 + (i))

// IRQ priorities
#define PRIO_MAIL   254
#define PRIO_PID    252
#define PRIO_UPD    251
#define PRIO_REP    241

// Deadlines to CLIC levels
#define TO_PRIO(dl_us)  (255U - ((dl_us) >> 8))

// CLIC software-pend
void riscv_clic_irq_set_pending(uint32_t irq);
#define clic_pend_irq(n)    riscv_clic_irq_set_pending(n)

/*
 * raise threshold, save old
 * re-enable global IRQs
 */
static inline unsigned int mintthresh_read(void)
{
    unsigned int val;
    __asm__ volatile("csrr %0, 0x347" : "=r"(val));
    return val;
}

// Writes val to mintthresh and restore previous value.
static inline unsigned int mintthresh_write(unsigned int val)
{
    unsigned int prev = mintthresh_read();
    __asm__ volatile("csrw 0x347, %0" :: "r"(val));
    return prev;
}

#endif /* BOARD_H */