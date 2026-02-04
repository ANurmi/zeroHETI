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

uint8_t tx_buf[BUF_BYTES] = {0};
uint8_t rx_buf[BUF_BYTES] = {0};

int main(void)
{
	printf("I2C Demonstrator on %s\n", CONFIG_BOARD_TARGET);

  i2c_init(PRESCALER);

  // set all enables in tx_buf
  tx_buf[0] = 0x0f;
  i2c_write_tx(SIM_CTRL_ADDR, tx_buf, BUF_BYTES);

  /*
  for (int i=0;i<BUF_BYTES;i++) {
    tx_buf[i] = 0xA0 + i;
  }

  i2c_write_tx(6, tx_buf, BUF_BYTES);
  i2c_read_tx(4, rx_buf, BUF_BYTES);

  uint32_t print_buf = 0;
  for (int i=0; i<BUF_BYTES;i++){
    print_buf |= ((uint32_t)rx_buf[i]) << i*8;
  }

  printf("rx_buf: 0x%8x\n", print_buf);
*/

  k_busy_wait(1);

  const uint32_t mbx_rd_addr  = 0x00030000;
  const uint32_t mbx_ack_addr = 0x00030004;
  /*
  const uint32_t mbx_tsl_addr = 0x00030008;
  const uint32_t mbx_tsh_addr = 0x0003000C;
  const uint32_t mbx_ms0_addr = 0x00030010;
  const uint32_t mbx_ms1_addr = 0x00030014;
  const uint32_t mbx_ms2_addr = 0x00030018;
  const uint32_t mbx_ms3_addr = 0x0003001C;
*/
  uint32_t data = sys_read32(mbx_rd_addr);

  // Test irq ack
  sys_write32(0x1, mbx_ack_addr);
  printf("Read %0x\n", data);

  debug_signal_pass();

	return 0;
}
