#ifndef BOARD_H
#define BOARD_H

/* Time to CPU ticks */
#define CPU_FREQ_HZ       10000000U
#define US_TO_TICKS(us)   ((us) * (CPU_FREQ_HZ / 1000000U))

/* I2C */
#define I2C_PRESCALER     4

/* APB Timers */
#define TIMER0_BASE       0x3300
#define TIMER1_BASE       0x3310
#define TIMER2_BASE       0x3320
#define TIMER3_BASE       0x3330

#define TIMER_CNT(base)   ((base) + 0x0)
#define TIMER_CTRL(base)  ((base) + 0x4)
#define TIMER_CMP(base)   ((base) + 0x8)

/* IRQ lines */
#define IRQ_TIMER0CMP     17
#define IRQ_TIMER1CMP     19
#define IRQ_TIMER2CMP     21
#define IRQ_TIMER3CMP     23
#define IRQ_MBX           26
#define IRQ_EXT0          27
#define IRQ_EXT1          28
#define IRQ_EXT2          29
#define IRQ_EXT3          30

/* IRQ priorities */
#define PRIO_TIMER_CMP    0x88
#define PRIO_EXT          0x10
#define PRIO_MBX          0x03

#endif