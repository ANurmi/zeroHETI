#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

use bsp::embedded_io::Write;
use bsp::hetic::{Hetic, Pol, Trig};
use bsp::interrupt::CoreInterrupt;
use bsp::rt::{InterruptNumber, core_interrupt};
use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    interrupt::ExternalInterrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mtimer::{self, MTimer},
    riscv::{self, asm::wfi},
    rt::entry,
    sprintln,
    tb::signal_pass,
    timer_group::{Periodic, Timer},
};
use core::arch::asm;
use fugit::ExtU32;
use more_asserts as ma;

#[cfg_attr(feature = "ufmt", derive(uDebug))]
#[cfg_attr(not(feature = "ufmt"), derive(Debug))]
struct Task {
    level: u8,
    period_ns: u32,
    duration_ns: u32,
}

const RUN_COUNT: usize = 1;
const TEST_DURATION: mtimer::Duration = mtimer::Duration::micros(1_000);

impl Task {
    pub const fn new(level: u8, period_ns: u32, duration_ns: u32) -> Self {
        Self {
            period_ns,
            duration_ns,
            level,
        }
    }
}

const TEST_BASE_PERIOD_NS: u32 = 100_000;
const TASK0: Task = Task::new(
    1,
    TEST_BASE_PERIOD_NS / 4,
    /* 25 ‰) */ TEST_BASE_PERIOD_NS / 40,
);
const TASK1: Task = Task::new(
    2,
    TEST_BASE_PERIOD_NS / 8,
    /* 12,5 ‰) */ TEST_BASE_PERIOD_NS / 80,
);
const TASK2: Task = Task::new(
    3,
    TEST_BASE_PERIOD_NS / 16,
    /* 5 ‰) */ TEST_BASE_PERIOD_NS / 200,
);
const TASK3: Task = Task::new(
    4,
    TEST_BASE_PERIOD_NS / 32,
    /* 2,5 ‰) */ TEST_BASE_PERIOD_NS / 400,
);
const CYCLES_PER_SEC: u64 = CPU_FREQ_HZ as u64;
const CYCLES_PER_MS: u64 = CYCLES_PER_SEC / 1_000;
const CYCLES_PER_US: u32 = CYCLES_PER_MS as u32 / 1_000;
// !!!: this would saturate to zero, so we must not use it. Use `X *
// CYCLES_PER_US / 1_000 instead` and verify the output value is not saturated.
/* const CYCLES_PER_NS: u64 = CYCLES_PER_US / 1_000; */

static mut TASK0_COUNT: usize = 0;
static mut TASK1_COUNT: usize = 0;
static mut TASK2_COUNT: usize = 0;
static mut TASK3_COUNT: usize = 0;
static mut TIMEOUT: bool = false;

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    //sprintln!("[periodic_tasks ({})]", env!("RISCV_EXTS"));
    sprintln!("[periodic_tasks]");
    sprintln!("Running test {} times", RUN_COUNT);

    sprintln!(
        "Tasks: \r\n  {:?}\r\n  {:?}\r\n  {:?}\r\n  {:?}",
        TASK0,
        TASK1,
        TASK2,
        TASK3
    );
    sprintln!(
        "Test duration: {} us ({} ns)",
        TEST_DURATION.to_micros(),
        TEST_DURATION.to_nanos()
    );

    setup_irq(ExternalInterrupt::Timer0Cmp, TASK0.level);
    setup_irq(ExternalInterrupt::Timer1Cmp, TASK1.level);
    setup_irq(ExternalInterrupt::Timer2Cmp, TASK2.level);
    setup_irq(ExternalInterrupt::Timer3Cmp, TASK3.level);
    setup_irq(CoreInterrupt::MachineTimer, u8::MAX);

    for run_idx in 0..RUN_COUNT {
        sprintln!("Run {}", run_idx);
        // SAFETY: interrupts off
        unsafe {
            TASK0_COUNT = 0;
            TASK1_COUNT = 0;
            TASK2_COUNT = 0;
            TASK3_COUNT = 0;
            TIMEOUT = false;

            // Make sure serial is done printing before proceeding to the test case
            serial.flush().unwrap_unchecked();
        }
        // Use mtimer for timeout
        let mut mtimer = MTimer::instance().into_oneshot();

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
            Timer::init::<TIMER3_ADDR>().into_periodic(),
        ];

        timers[0].set_period(TASK0.period_ns.nanos());
        timers[1].set_period(TASK1.period_ns.nanos());
        timers[2].set_period(TASK2.period_ns.nanos());
        timers[3].set_period(TASK3.period_ns.nanos());

        // --- Test critical ---
        unsafe {
            asm!("fence");
            // clear mcycle, minstret at start of critical section
            asm!("csrw 0xB00, {0}", in(reg) 0x0);
            asm!("csrw 0xB02, {0}", in(reg) 0x0);
            /* !!! mcycle and minstret are missing write-methods in BSP !!! */
        };

        // Test will end when MachineTimer fires
        mtimer.start(TEST_DURATION);

        // Start periodic timers
        timers.iter_mut().for_each(Periodic::start);

        unsafe { riscv::interrupt::enable() };

        while !unsafe { TIMEOUT } {
            wfi();
        }

        riscv::interrupt::disable();
        unsafe { asm!("fence") };
        // --- Test critical end ---

        unsafe {
            let mcycle = riscv::register::mcycle::read64();
            let minstret = riscv::register::minstret::read64();

            sprintln!("cycles: {}", mcycle);
            sprintln!("instrs: {}", minstret);
            sprintln!(
                "Task counts:\r\n{} | {} | {} | {}",
                TASK0_COUNT,
                TASK1_COUNT,
                TASK2_COUNT,
                TASK3_COUNT
            );
            let total_ns_in_task0 = TASK0.duration_ns * TASK0_COUNT as u32;
            let total_ns_in_task1 = TASK1.duration_ns * TASK1_COUNT as u32;
            let total_ns_in_task2 = TASK2.duration_ns * TASK2_COUNT as u32;
            let total_ns_in_task3 = TASK3.duration_ns * TASK3_COUNT as u32;
            sprintln!(
                "Theoretical total duration spent in task workload (ns):\r\n{} | {} | {} | {} = {}",
                total_ns_in_task0,
                total_ns_in_task1,
                total_ns_in_task2,
                total_ns_in_task3,
                total_ns_in_task0 + total_ns_in_task1 + total_ns_in_task2 + total_ns_in_task3,
            );

            // Assert that each task runs the expected number of times
            for (count, task) in &[
                (TASK0_COUNT, TASK0),
                (TASK1_COUNT, TASK1),
                (TASK2_COUNT, TASK2),
                (TASK3_COUNT, TASK3),
            ] {
                // Assert task count is at least the expected count. There may be one less as
                // the final in-flight task might get interrupted by the test
                // end.
                ma::assert_ge!(
                    *count,
                    (TEST_DURATION.to_nanos() as usize / task.period_ns as usize) - 1
                );
                ma::assert_le!(
                    *count,
                    (TEST_DURATION.to_nanos() as usize / task.period_ns as usize)
                );
            }

            // Make sure serial is done printing before proceeding to the next iteration
            serial.flush().unwrap_unchecked();
        }
    }

    // Clean up
    tear_irq(ExternalInterrupt::Timer0Cmp);
    tear_irq(ExternalInterrupt::Timer1Cmp);
    tear_irq(ExternalInterrupt::Timer2Cmp);
    tear_irq(ExternalInterrupt::Timer3Cmp);
    tear_irq(CoreInterrupt::MachineTimer);

    signal_pass(Some(&mut serial));
    loop {
        // Wait for interrupt
        wfi();
    }
}

// This gets pasted for each inline handler if `-Finline-isrs` is defined
macro_rules! impl_inline_isr {
    ($irq:expr, $TASK_COUNT:expr, $TASK:expr) => {
        core::arch::global_asm!(
            concat!(
            r#"
            .section .trap, "ax"
            .align 4
            "#,
            concat!(".global _start_", $irq, "_trap\n"),
            concat!("_start_", $irq, "_trap:\n"),
            r#"
                #----- Interrupts disabled on entry ---#
                csrsi mstatus, 8    // enable interrupts
                #----- Interrupts enabled -------------#

                // Increment CNT
                lla     a0, {CNT}
                lw      a1, 0(a0)
                addi    a1,a1,1
                sw      a1, 0(a0)

                // NOP workload
                .rept {NOP_CNT}
                nop
                .endr

                csrci mstatus, 8    // disable interrupts
                #----- Interrupts disabled  ---------#
                mret
            "#
            ), CNT = sym $TASK_COUNT, NOP_CNT = const $TASK.duration_ns * CYCLES_PER_US / 1_000
        );
    };
}

// Nested PCS interrupt in assembly (Timer0Cmp)
impl_inline_isr!("Timer0Cmp", TASK0_COUNT, TASK0);

// Nested PCS interrupt in assembly (Timer1Cmp)
impl_inline_isr!("Timer1Cmp", TASK1_COUNT, TASK1);

// Nested PCS interrupt in assembly (Timer2Cmp)
impl_inline_isr!("Timer2Cmp", TASK2_COUNT, TASK2);

// Nested PCS interrupt in assembly (Timer3Cmp)
impl_inline_isr!("Timer3Cmp", TASK3_COUNT, TASK3);

/// Timeout interrupt (per test-run)
#[core_interrupt(bsp::interrupt::CoreInterrupt::MachineTimer)]
unsafe fn MachineTimer() {
    unsafe { TIMEOUT = true };
    let mut timer = MTimer::instance();
    timer.disable();

    // Draw mtimer to max value to make sure all currently pending or in flight
    // TimerXCmp interrupts fall through.
    unsafe { timer.set_counter(u64::MAX) };

    // Disable all timers & interrupts, so no more instances will fire
    unsafe {
        Timer::instance::<TIMER0_ADDR>().disable();
        Timer::instance::<TIMER1_ADDR>().disable();
        Timer::instance::<TIMER2_ADDR>().disable();
        Timer::instance::<TIMER3_ADDR>().disable()
    };
    Hetic::line(CoreInterrupt::MachineTimer.number()).unpend();
    Hetic::line(ExternalInterrupt::Timer0Cmp.number()).unpend();
    Hetic::line(ExternalInterrupt::Timer1Cmp.number()).unpend();
    Hetic::line(ExternalInterrupt::Timer2Cmp.number()).unpend();
    Hetic::line(ExternalInterrupt::Timer3Cmp.number()).unpend();
}

pub fn setup_irq(irq: impl InterruptNumber, level: u8) {
    Hetic::line(irq.number()).set_trig(Trig::Edge);
    Hetic::line(irq.number()).set_pol(Pol::Pos);
    Hetic::line(irq.number()).set_level(level);
    Hetic::line(irq.number()).enable();
}

/// Tear down the IRQ configuration to avoid side-effects for further testing
pub fn tear_irq(irq: impl InterruptNumber) {
    Hetic::line(irq.number()).disable();
    Hetic::line(irq.number()).set_level(0x0);
    Hetic::line(irq.number()).set_trig(Trig::Level);
    Hetic::line(irq.number()).set_pol(Pol::Pos);
}
