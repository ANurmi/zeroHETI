/*
 * SPDX-License-Identifier: Apache-2.0
 *
 * Zephyr software-EDF.
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

// Machine Timer
#define MTIMER_BASE  0x3100U
#define MTIMER_CTRL  (MTIMER_BASE + 0x10U)

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

#define THREAD_PRIO  5
#define THREAD_STACK 1024

static struct k_sem job_sem[NUM_TASKS];
static struct k_thread thread_th[NUM_TASKS];
static K_THREAD_STACK_ARRAY_DEFINE(thread_stacks, NUM_TASKS, THREAD_STACK);

static volatile uint32_t irq_count[NUM_TASKS];
static uint32_t jobs_done[NUM_TASKS];
static volatile uint8_t run_done;


#define FINISH_TIMER   3
#define FINISH_IRQ     IRQ_TIMER_CMP(FINISH_TIMER)
// Most urgent task 
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

// Burns a fixed number of instructions
static void spin(uint32_t iters)
{
	for (uint32_t i = 0; i < iters; i++) {
		__asm__ volatile("nop");
	}
}

static uint32_t job_iters[NUM_TASKS];

static void isr_release(const void *arg)
{
	const int n = (int)(uintptr_t)arg;
	uint32_t now = k_cycle_get_32();

	irq_count[n]++;

	k_thread_absolute_deadline_set(&thread_th[n],
	(int)(now + US_TO_MTIME(task_set[n].deadline_us)));

	k_sem_give(&job_sem[n]);
}

static void isr_finish(const void *arg)
{
	ARG_UNUSED(arg);

	// Scoreboard disable
	sys_write32(0U, SCB_ENABLE);
	mintthresh_write(0xFF); 
	run_done = 1U;
}

static void thread(void *a, void *b, void *c)
{
	const int n = (int)(uintptr_t)a;

	ARG_UNUSED(b);
	ARG_UNUSED(c);

	while (1) {
		k_sem_take(&job_sem[n], K_FOREVER);

		sys_write32(1U, SCB_TASK(n));
		spin(job_iters[n]);
		sys_write32(0U, SCB_TASK(n));

		jobs_done[n]++;
	}
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
	uint32_t utilization;
	uint64_t start_cycles;
	uint32_t expected;
	uint32_t hyperperiod = lcm(lcm(task_set[0].period_us, task_set[1].period_us),
			   					   task_set[2].period_us);
	bool is_edfic = (platform & PLATFORM_INTC_EDFIC) != 0U;

	if (is_edfic) {
		sys_write32(0x1, CFG_MTIME_EN);
	}

	printf("[micro-rtprof/zephyr] interrupt controller microbenchmark\n");

	printf("Platform - HW commit   : %x, intc: %s,        CPU Frequency (MHz): %u\n",
	       commit, is_edfic ? "EDFIC" : "CLIC ", CPU_FREQ_HZ / 1000000U);

	printf("Testcase - runtime (ms): %7u, load: (0..100): %u,    Hyperperiod (us): %u\n",
	       (uint32_t)RUNTIME_MS, (uint32_t)LOAD_FACTOR, hyperperiod);

	printf("mtime    - ctrl: %04x, prescaler: %u, %u tick/us, mtime_en: %u\n",
	       sys_read32(MTIMER_CTRL), (uint32_t)CONFIG_ZEROHETI_MTIME_PRESCALER,
	       MTIME_HZ / 1000000U, sys_read32(CFG_MTIME_EN));

	for (int i = 0; i < NUM_TASKS; i++) {
		uint32_t jobs_t = hyperperiod / task_set[i].period_us;
		uint32_t C_t = jobs_t * task_set[i].C_us;

		C_total += C_t;

		printf("Task %d: T: %3u us, D: %3u us, C: %2u us, jobs (per HP): %3u, C_t: %4u us\n",
		       i, task_set[i].period_us, task_set[i].deadline_us,
		       task_set[i].C_us, jobs_t, C_t);
	}

	utilization = (C_total * 100U) / hyperperiod;
	printf("Theoretical CPU utilization: C_total %u us / %u us = %u %%\n",
	       C_total, hyperperiod, utilization);

	printf("\nZephyr EDF, %u threads @ prio %d, for %u ms\n",
	       NUM_TASKS, THREAD_PRIO, (uint32_t)RUNTIME_MS);

	const uint32_t n = 4096U;
	unsigned int key = irq_lock();
	uint32_t t0, t1, cc;

	// first spin for callibration. 
	spin(n);
	t0 = read_mcycle();
	spin(n);
	t1 = read_mcycle();
	irq_unlock(key);
	cc = t1 - t0;

	// measure how many iterations for C_tx
	for (int i = 0; i < NUM_TASKS; i++) {
		uint32_t target = US_TO_TICKS(task_set[i].C_us);
		job_iters[i] = (uint32_t)(((uint64_t)target * n) / cc);
	}
	printf("spin: %u iters in %u cc = %u.%02u cc/iter, %u iters/us\n",
	       n, cc, cc / n, ((cc % n) * 100U) / n,
	       (n * US_TO_TICKS(1)) / cc);
	printf("C0 %u us = %u iters, C1 %u us = %u iters, C2 %u us = %u iters\n",
	       task_set[0].C_us, job_iters[0], task_set[1].C_us, job_iters[1],
	       task_set[2].C_us, job_iters[2]);
	

	// Check the calibration against an independent mcycle read
	{
		unsigned int key = irq_lock();
		uint32_t t0 = read_mcycle();

		spin(job_iters[0]);

		uint32_t cc = read_mcycle() - t0;

		irq_unlock(key);
		printf("C0 measured: %u cc (want %u cc = %u us)\n",
		       cc, US_TO_TICKS(task_set[0].C_us), task_set[0].C_us);
	}

	// Setup IRQ
	IRQ_CONNECT(IRQ_TIMER_CMP(0), 0, isr_release, (void *)0, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(1), 0, isr_release, (void *)1, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(2), 0, isr_release, (void *)2, 0);
	IRQ_CONNECT(IRQ_TIMER_CMP(3), 0, isr_finish, NULL, 0);

	// Block all interrupts
	mintthresh_write(0xFF);

	for (int i = 0; i < NUM_TASKS; i++) {
		edfic_setup(IRQ_TIMER_CMP(i), US_TO_MTIME(task_set[i].deadline_us));

		k_sem_init(&job_sem[i], 0, 1);
		k_thread_create(&thread_th[i], thread_stacks[i], THREAD_STACK, thread,
				(void *)(uintptr_t)i, NULL, NULL, THREAD_PRIO, 0, K_NO_WAIT);

		send_letter(SCB_DL_ADDR(i), US_TO_TICKS(task_set[i].deadline_us));
	}

	edfic_setup(FINISH_IRQ, US_TO_MTIME(FINISH_DL_US));

	k_thread_priority_set(k_current_get(), K_LOWEST_APPLICATION_THREAD_PRIO);

	// Scoreboard enable
	sys_write32(1U, SCB_ENABLE);
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

	while (!run_done) {
	}

	printf("run: %u mtime ticks (want %u)\n",
	       (uint32_t)(k_cycle_get_64() - start_cycles),
	       (uint32_t)US_TO_MTIME(RUNTIME_MS * 1000U));

	for (int i = 0; i < NUM_TASKS; i++) {
		expected = (RUNTIME_MS * 1000U) / task_set[i].period_us;
		printf("T%d: releases: %u (expected ~%u), jobs run: %u, dropped: %u\n",
		       i, irq_count[i], expected, jobs_done[i],
		       irq_count[i] - jobs_done[i]);
	}

	debug_signal_pass();

	return 0;
}
