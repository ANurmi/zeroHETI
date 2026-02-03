/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <debug/debug.h>
#include <i2c/i2c.h>

#define PRESCALER 4

#define BUF_BYTES 4

uint8_t tx_buf[BUF_BYTES] = {0};
uint8_t rx_buf[BUF_BYTES] = {0};

int main(void)
{
	printf("I2C Demonstrator on %s\n", CONFIG_BOARD_TARGET);

  i2c_init(PRESCALER);

  for (int i=0;i<BUF_BYTES;i++) {
    tx_buf[i] = 0xA0 + i;
  }

  i2c_write_tx(6, tx_buf, 3);
  i2c_read_tx(4, rx_buf, 4);

  for (int i=0; i<BUF_BYTES;i++){
    printf("rx_buf[%0d]: 0x%x\n", i, rx_buf[i]);
  }

  debug_signal_pass();

	return 0;
}
