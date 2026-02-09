/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <debug/debug.h>
#include <zephyr/kernel.h>
#include <i2c/i2c.h>

#define PRESCALER 4

#define BUF_BYTES 4

#define SIM_CTRL_ADDR 0

#define MBX_INBOX_ADDR   0x30000
#define MBX_IRQ_ACK_ADDR 0x30004
#define MBX_TIME_LO_ADDR 0x30008
#define MBX_TIME_HI_ADDR 0x3000C
#define MBX_M0_STAT_ADDR 0x30010
#define MBX_M1_STAT_ADDR 0x30014
#define MBX_M2_STAT_ADDR 0x30018
#define MBX_M3_STAT_ADDR 0x3001C

uint8_t tx_buf[BUF_BYTES] = {0};
uint8_t rx_buf[BUF_BYTES] = {0};

int main(void)
{
	printf("I2C Demonstrator on %s\n", CONFIG_BOARD_TARGET);

  i2c_init(PRESCALER);

  // Start sim by writing to SIM_CTRL_ADDR
  tx_buf[0] = 0x1;
  i2c_write_tx(SIM_CTRL_ADDR, tx_buf, BUF_BYTES);

  //k_busy_wait(600);
  k_busy_wait(400);

  uint32_t data = sys_read32(MBX_INBOX_ADDR);

  // Update M0-3 status
  sys_write32(0x1, MBX_M0_STAT_ADDR);
  sys_write32(0x1, MBX_M1_STAT_ADDR);
  sys_write32(0x1, MBX_M2_STAT_ADDR);
  sys_write32(0x1, MBX_M3_STAT_ADDR);

  printf("Read data: 0x%x\n", data);

  k_busy_wait(60);
  
  // Update M0-3 status
  sys_write32(0x1, MBX_M0_STAT_ADDR);
  sys_write32(0x1, MBX_M1_STAT_ADDR);
  sys_write32(0x1, MBX_M2_STAT_ADDR);
  sys_write32(0x1, MBX_M3_STAT_ADDR);

  // Test irq ack
  sys_write32(0x1, MBX_IRQ_ACK_ADDR);

  debug_signal_pass();

	return 0;
}
