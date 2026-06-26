#ifndef MAILBOX_H
#define MAILBOX_H

#include <zephyr/sys/sys_io.h>

/* --- Hardware mailbox registers --- */
/* If MBX_STAT [2] is true - safe to send, if [1] is true - inbox is full */
#define MBX_STAT    0x00030000 
#define MBX_CTRL    0x00030004
#define MBX_IADD    0x0003000C
#define MBX_IDAT    0x00030010
#define MBX_OADD    0x00030014
#define MBX_ODAT    0x00030018

#define MBX_CTRL_SEND       0x00000001
#define MBX_CTRL_INBOX_ACK  0x01000000
#define MBX_CTRL_IRQ_CLR    0x00020000

/* Write outbox letters */
static inline void send_letter(uint32_t addr, uint32_t data)
{   
    /* Check MBX_STAT [2] bit */
    while (!(sys_read32(MBX_STAT) & (1u << 2)))
        ;
    sys_write32(addr, MBX_OADD);
    sys_write32(data, MBX_ODAT);
    sys_write32(MBX_CTRL_SEND, MBX_CTRL);
}

/* Read and ack inbox letters */
static inline void read_letter(uint32_t *addr, uint32_t *data)
{
    *addr = sys_read32(MBX_IADD);
    *data = sys_read32(MBX_IDAT);
    sys_write32(MBX_CTRL_INBOX_ACK, MBX_CTRL);
}

/* Clear the mailbox IRQ line after reading inbox letters. */
static inline void mbx_clear_irq(void)
{
    sys_write32(MBX_CTRL_IRQ_CLR, MBX_CTRL);
}

/* --- VIP sim-control addresses --- */

#define SIM_START       0x01000000
#define SIM_STOP        0x01000001
#define SIM_PRESCALER   0x01000002
#define SIM_LOADFACTOR  0x01000003

/* --- Tasks address map and structure --- */
/*
 * Tasks start at 0x02000000 each occupies 0x10000.
 * Within each slot: (0) period, +(1) deadline, +(2) ack/pend.
 *  
 * idx is task number
 * n is a motor number
 */
#define TASK_BASE(idx)      (0x02000000 + (idx) * 0x10000)
#define TASK_PERIOD(idx)    (TASK_BASE(idx) + 0)
#define TASK_DEADLINE(idx)  (TASK_BASE(idx) + 1)
#define TASK_ACK(idx)       (TASK_BASE(idx) + 2)

#define TASK_MBX        0
#define TASK_UPD(n)     (1 + (n))
#define TASK_CTRL(n)    (5 + (n))
#define TASK_REP(n)     (9 + (n))

/* Report */
#define MBX_PRINT_ADDR  0x03000000

#endif /* MAILBOX_H */