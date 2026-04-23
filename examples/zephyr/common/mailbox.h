#ifndef MAILBOX_H
#define MAILBOX_H

#define MBX_INBOX_ADDR      0x30000
#define MBX_IRQ_ACK_ADDR    0x30004
#define MBX_TIME_LO_ADDR    0x30008
#define MBX_TIME_HI_ADDR    0x3000C
#define MBX_M0_STAT_ADDR    0x30010
#define MBX_M1_STAT_ADDR    0x30014
#define MBX_M2_STAT_ADDR    0x30018
#define MBX_M3_STAT_ADDR    0x3001C

static const uint32_t mbx_stat_addr[4] = {
    MBX_M0_STAT_ADDR, MBX_M1_STAT_ADDR, 
    MBX_M2_STAT_ADDR, MBX_M3_STAT_ADDR
};

#endif

