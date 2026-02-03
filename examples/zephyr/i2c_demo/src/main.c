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

uint8_t* tx_buf = NULL;
uint8_t* rx_buf = NULL;

int main(void)
{
	printf("I2C Demonstrator on %s\n", CONFIG_BOARD_TARGET);

  i2c_init(PRESCALER);

  for (int i=0;i<4;i++) {
    *tx_buf = 0xA0 + i;
    tx_buf++;
  }

  i2c_write_tx(6, tx_buf);
  rx_buf = i2c_read_tx(4);

  if (rx_buf == NULL) {
    printf("rx_buf empty\n");
  } else {
    printf("rx_buf: %x\n", rx_buf[0]);
  };
  
  debug_signal_pass();

	return 0;
}
