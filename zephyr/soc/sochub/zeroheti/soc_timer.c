#include <zephyr/init.h>
#include <zephyr/arch/cpu.h>
#include <zephyr/sys/sys_io.h>

#define MTIMER_CTRL 0x3110U

static int start_mtimer(void)
{
	sys_write32((CONFIG_ZEROHETI_MTIME_PRESCALER << 8) | 0x1U, MTIMER_CTRL);
	return 0;
}

SYS_INIT(start_mtimer, EARLY, 0);
