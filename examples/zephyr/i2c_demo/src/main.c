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

#define I2C_SIM_CTRL_ADDR 0
#define I2C_M0_STAT_ADDR  2
#define I2C_M0_CTRL_ADDR  3
#define I2C_M1_STAT_ADDR  4
#define I2C_M1_CTRL_ADDR  5
#define I2C_M2_STAT_ADDR  6
#define I2C_M2_CTRL_ADDR  7
#define I2C_M3_STAT_ADDR  8
#define I2C_M3_CTRL_ADDR  9

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
  i2c_write_tx(I2C_SIM_CTRL_ADDR, tx_buf, BUF_BYTES);

  //k_busy_wait(600);
  k_busy_wait(400);

  uint32_t data = sys_read32(MBX_INBOX_ADDR);

  // Update M0-3 status
  sys_write32(0x1, MBX_M0_STAT_ADDR);
  sys_write32(0x1, MBX_M1_STAT_ADDR);
  sys_write32(0x1, MBX_M2_STAT_ADDR);
  sys_write32(0x1, MBX_M3_STAT_ADDR);

  printf("MBX Read data: 0x%x\n", data);

  k_busy_wait(60);
  
  // Update M0-3 status
  sys_write32(0x1, MBX_M0_STAT_ADDR);
  sys_write32(0x1, MBX_M1_STAT_ADDR);
  sys_write32(0x1, MBX_M2_STAT_ADDR);
  sys_write32(0x1, MBX_M3_STAT_ADDR);

  // Test irq ack
  sys_write32(0x1, MBX_IRQ_ACK_ADDR);

  tx_buf[0] = 0x00;
  tx_buf[1] = 0x10;
  tx_buf[2] = 0x01;
  tx_buf[3] = 0x00;

  i2c_write_tx(I2C_M0_CTRL_ADDR, tx_buf, BUF_BYTES);
  
  k_busy_wait(100);
  i2c_read_tx(I2C_M0_STAT_ADDR, rx_buf, BUF_BYTES);

  uint32_t m0_speed = 0;
  m0_speed         |= ((uint32_t)rx_buf[3]) << 24;
  m0_speed         |= ((uint32_t)rx_buf[2]) << 16;
  m0_speed         |= ((uint32_t)rx_buf[1]) << 8;
  m0_speed         |= ((uint32_t)rx_buf[0]) << 0;
  printf("M0 Speed: %0d\n", m0_speed);

  debug_signal_pass();

	return 0;
}
