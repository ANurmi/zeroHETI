#include <zephyr/init.h>
#include <zephyr/arch/cpu.h>

static int start_mtimer(void) {
	// Enable mtimer with prescaler 4
	//sys_write32(0x401, 0x2110);
	// Enable mtimer w/o prescaler
	sys_write32(0x1, 0x2110);
	return 0;
}

SYS_INIT(start_mtimer, EARLY, 0);
