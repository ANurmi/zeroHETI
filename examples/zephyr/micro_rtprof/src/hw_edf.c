/*
 * SPDX-License-Identifier: Apache-2.0
 *
 * Zephyr hardware-EDF(EDFIC).
 */
#include <stdio.h>
#include <stdint.h>
#include <zephyr/kernel.h>
#include <zephyr/irq.h>
#include <zephyr/sys/sys_io.h>
#include <debug/debug.h>
#include "board.h"
#include "mailbox.h"

// apb_cfg_regs
#define CFG_BASE            0x00004000U
#define CFG_COMMIT          (CFG_BASE + 0x000U)
#define CFG_PLATFORM        (CFG_BASE + 0x004U)
#define CFG_MTIME_EN        (CFG_BASE + 0x008U)
#define PLATFORM_INTC_EDFIC (1U << 0)

// Scoreboard: gpreg[0]
#define SCB_ENABLE     (CFG_BASE + 0x100U)
#define SCB_TASK(i)    (CFG_BASE + 0x104U + 4U * (i))
#define SCB_DL_ADDR(i) (0x00010000U + (i))

// EDFIC
#define INTC_BASE      0x00100000U
#define EDFIC_LINE(n)  (INTC_BASE + 4U * (n))
#define EDFIC_IE       (1U << 0)

#define PRESCALER (CONFIG_ZEROHETI_MTIME_PRESCALER + 1U)
#define MTIME_HZ  (CPU_FREQ_HZ / PRESCALER)
#define US_TO_MTIME(us) ((us) * (MTIME_HZ / 1000000U))

#define NUM_TASKS 3

struct rt_task {
	uint32_t period_us;
	uint32_t deadline_us;
	uint32_t C_us;
};

static const struct rt_task task_set[NUM_TASKS] = {
	{ .period_us =  30, .deadline_us =  50, .C_us =  8U * LOAD_FACTOR / 100U },
	{ .period_us =  66, .deadline_us = 100, .C_us = 30U * LOAD_FACTOR / 100U },
	{ .period_us = 170, .deadline_us = 150, .C_us = 50U * LOAD_FACTOR / 100U },
};

static struct k_sem run_done;

#define FINISH_TIMER   3
#define FINISH_IRQ     IRQ_TIMER_CMP(FINISH_TIMER)
// Most urgent line
#define FINISH_DL_US   10U

static inline void edfic_setup(uint32_t irq, uint32_t dl)
{
	sys_write32((dl << 8) | EDFIC_IE, EDFIC_LINE(irq));
}

static inline uint32_t read_mcycle(void)
{
	uint32_t buf;
	__asm__ volatile("csrr %0, mcycle" : "=r"(buf));
	return buf;
}

static inline uint32_t read_minstret(void)
{
	uint32_t buf;
	__asm__ volatile("csrr %0, minstret" : "=r"(buf));
	return buf;
}

// Burns a fixed number of instructions.
static void __attribute__((noinline)) spin(uint32_t iters)
{
	for (uint32_t i = 0; i < iters; i++) {
		__asm__ volatile("nop");
	}
}

static uint32_t job_iters[NUM_TASKS];

static void isr_release(const void *arg)
{
	const int n = (int)(uintptr_t)arg;

	__asm__ volatile("csrsi mstatus, 0x8");

	sys_write32(1U, SCB_TASK(n));
	spin(job_iters[n]);
	sys_write32(0U, SCB_TASK(n));

	__asm__ volatile("csrci mstatus, 0x8");
}

static void isr_finish(const void *arg)
{
	ARG_UNUSED(arg);

	// Scoreboard disable
	sys_write32(0U, SCB_ENABLE);
	mintthresh_write(0xFF);
	k_sem_give(&run_done);
}

static uint32_t gcd(uint32_t a, uint32_t b)
{
	while (b != 0U) {
		uint32_t r = a % b;

		a = b;
		b = r;
	}
	return a;
}

static uint32_t lcm(uint32_t a, uint32_t b)
{
	return ((a == 0U) || (b == 0U)) ? 0U : (a / gcd(a, b)) * b;
}

int main(void)
{
	uint32_t commit = sys_read32(CFG_COMMIT);
	uint32_t platform = sys_read32(CFG_PLATFORM);
	uint32_t C_total = 0U;
	uint32_t C_t[NUM_TASKS];
	uint32_t utilization;
	uint64_t start_cycles;
	uint64_t total_cc;
	uint32_t active_cc, instret;
	uint32_t hyperperiod = lcm(lcm(task_set[0].period_us, task_set[1].period_us),
				   task_set[2].period_us);
	bool is_edfic = (platform & PLATFORM_INTC_EDFIC) != 0U;

	if (is_edfic) {
		sys_write32(0x1, CFG_MTIME_EN);
	}

	printf("[micro-rtprof] interrupt controller microbenchmark\n");

	printf("Platform - HW commit   : %x, intc: %s,        CPU Frequency (MHz): %u\n",
	       commit, is_edfic ? "EDFIC" : "CLIC ", CPU_FREQ_HZ / 1000000U);

	printf("Testcase - runtime (ms): %7u, load: (0..100): %u,    Hyperperiod (us): %u\n",
	       (uint32_t)RUNTIME_MS, (uint32_t)LOAD_FACTOR, hyperperiod);

	for (int i = 0; i < NUM_TASKS; i++) {
		uint32_t jobs_t = hyperperiod / task_set[i].period_us;

		C_t[i] = jobs_t * task_set[i].C_us;
		C_total += C_t[i];

		printf("Task %d: F (per HP): %u, Total runtime (us): %u\n", i, jobs_t, C_t[i]);
	}

	utilization = (C_total * 100U) / hyperperiod;
	printf("Theoretical CPU utilization: %u + %u + %u us/%u us = %u %% \n\n",
	       C_t[0], C_t[1], C_t[2], hyperperiod, utilization);

	// How many iterations in spin() make up each task's C
	{
		const uint32_t n = 4096U;
		unsigned int key = irq_lock();
		uint32_t t0, t1, cc;

		spin(n);
		t0 = read_mcycle();
		spin(n);
		t1 = read_mcycle();
		irq_unlock(key);

		cc = t1 - t0;

		for (int i = 0; i < NUM_TASKS; i++) {
			uint32_t target = US_TO_TICKS(task_set[i].C_us);

			job_iters[i] = (uint32_t)(((uint64_t)target * n) / cc);
		}
	}

	// Setup IRQ
	IRQ_CONNECT(IRQ_TIMER_CMP(0), 0, isr_release, (void *)0, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(1), 0, isr_release, (void *)1, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(2), 0, isr_release, (void *)2, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(3), 0, isr_finish, NULL, 0);

	k_sem_init(&run_done, 0, 1);

	// Block all interrupts
	mintthresh_write(0xFF);

	for (int i = 0; i < NUM_TASKS; i++) {
		edfic_setup(IRQ_TIMER_CMP(i), US_TO_MTIME(task_set[i].deadline_us));

		send_letter(SCB_DL_ADDR(i), US_TO_TICKS(task_set[i].deadline_us));
	}

	edfic_setup(FINISH_IRQ, US_TO_MTIME(FINISH_DL_US));

	k_thread_priority_set(k_current_get(), K_LOWEST_APPLICATION_THREAD_PRIO);

	// Scoreboard enable
	sys_write32(1U, SCB_ENABLE);

	__asm__ volatile("csrw mcycle,    0");
	__asm__ volatile("csrw mcycleh,   0");
	__asm__ volatile("csrw minstret,  0");
	__asm__ volatile("csrw minstreth, 0");

	start_cycles = k_cycle_get_64();

	// Config and start timers
	for (int i = 0; i < NUM_TASKS; i++) {
		sys_write32(US_TO_TICKS(task_set[i].period_us), TIMER_CMP(TIMER_BASE(i)));
		sys_write32(0x1, TIMER_CTRL(TIMER_BASE(i)));
	}
	sys_write32(US_TO_TICKS(RUNTIME_MS * 1000U), TIMER_CMP(TIMER_BASE(FINISH_TIMER)));
	sys_write32(0x1, TIMER_CTRL(TIMER_BASE(FINISH_TIMER)));

	// Release
	mintthresh_write(0x00);

	k_sem_take(&run_done, K_FOREVER);

	active_cc = read_mcycle();
	instret = read_minstret();
	total_cc = (k_cycle_get_64() - start_cycles) * PRESCALER;

	printf("True CPU utilization: %u %%, instructions: %u\n",
	       (uint32_t)(((uint64_t)active_cc * 100U) / total_cc), instret);

	debug_signal_pass();

	return 0;
}
