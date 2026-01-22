/*
 * Copyright (c) 2012-2014 Wind River Systems, Inc.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>
#include <debug/debug.c>

extern int debug_signal_pass();

int main(void)
{
	printf("Hello World! %s\n", CONFIG_BOARD_TARGET);

  debug_signal_pass();

	return 0;
}
