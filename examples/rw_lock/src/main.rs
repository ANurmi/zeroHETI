#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;

#[cfg_attr(feature = "obs", rtic::app(device = bsp, obs = obs_trace::Obs, dispatchers = []))]
#[cfg_attr(not(feature = "obs"), rtic::app(device = bsp, dispatchers = []))]
mod app {
    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        clear_perf_counters,
        fugit::ExtU32,
        lcm,
        mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mtimer::{Duration32, MTimerLo, OneShot},
        parse_u32,
        register::{mcycle, minstret},
        sprintln,
        tb::signal_pass,
        timer_group::Timer,
    };
    use core::mem::MaybeUninit;

    // # Hyperparams

    /// Runtime in milliseconds, read from env `RUNTIME_MS`
    const RUNTIME_MS: u32 = parse_u32(env!("RUNTIME_MS"));
    const LOCK_MODE: &str = if cfg!(feature = "rw") {
        "rw-lock"
    } else {
        "mutex"
    };

    /// Period of `ReaderHigh`
    const PERIOD_RHI_US: u32 = 100;
    /// Period of `J`
    const PERIOD_J_US: u32 = 200;
    /// Period of `ReaderLow`
    const PERIOD_RLO_US: u32 = 300;
    /// Period of `W`
    const PERIOD_W_US: u32 = 500;

    /// Duration of the critical section of `ReaderHigh`. Short.
    const CS_RHI: Duration32 = Duration32::from_micros(10);
    /// Duration of the work of `J`.
    const WORK_J: Duration32 = Duration32::from_micros(40);
    /// Duration of the critical section of `ReaderLow`. Long.
    const CS_RLO: Duration32 = Duration32::from_micros(150);
    /// Duration of the critical section of `W`. Short.
    const CS_W: Duration32 = Duration32::from_micros(10);

    const DL_J_US: u32 = 100;

    /// Set to trigger all timers in given time. Timer periods are shifted in
    /// phase the same amount.
    const PRE_TRIGGER: Option<Duration32> = Some(Duration32::from_micros(10));

    const HYPERPERIOD_US: u32 = lcm!(PERIOD_RHI_US, PERIOD_J_US, PERIOD_RLO_US, PERIOD_W_US);
    const TICKS_PER_US: u32 = CPU_FREQ_HZ / 1_000_000;

    // Calculate theoretical load percent
    const UTIL_MIN_PC: u32 = CS_RHI.as_ticks() * 100 / (PERIOD_RHI_US * TICKS_PER_US)
        + WORK_J.as_ticks() * 100 / (PERIOD_J_US * TICKS_PER_US)
        + CS_RLO.as_ticks() * 100 / (PERIOD_RLO_US * TICKS_PER_US)
        + CS_W.as_ticks() * 100 / (PERIOD_W_US * TICKS_PER_US);

    // # Global records

    /// Per task job statistics
    #[derive(Clone, Copy)]
    pub struct TaskStat {
        id: &'static str,
        prio: u16,
        /// Period of the associated task
        period: Duration32,
        /// Deadline of the task, usually same as period.
        dl: Duration32,
        /// Worst observed response time,
        /// disregarding deadline overshoots
        worst: Duration32,
        /// Has a deadline miss been observed?
        miss_count: usize,
        /// Number of completed jobs
        count: u32,
        /// Has a spurious interrupt been recorded?
        spurious_count: usize,
        last_spurious: Duration32,
    }
    impl TaskStat {
        fn new(id: &'static str, prio: u16, period: Duration32, dl: Duration32) -> TaskStat {
            TaskStat {
                id,
                prio,
                period,
                dl,
                worst: Duration32::ZERO,
                miss_count: 0,
                count: 0,
                spurious_count: 0,
                last_spurious: Duration32::ZERO,
            }
        }

        fn print_stats(runtime_measured: &Duration32, stats: &[&Self]) {
            // Sanity-check the recorded completions against the number of
            // releases that can actually have completed. A task release occurs
            // at `SYS_START + k*period`; `Teardown` runs at the highest priority
            // and its `MachineTimer` fires at the runtime boundary, so the
            // release coinciding with the boundary itself never completes.
            // Hence completions may trail the raw release count by one job.
            for TaskStat {
                id,
                period,
                count,
                spurious_count,
                last_spurious,
                ..
            } in stats
            {
                const TAG_WARN: &str = "[WARN]";
                // Note that `runtime_measured` is usually slightly less than
                // actual runtime, but not always. Subtract a microsecond to
                // make sure the formula for expected job count remains stable.
                let expected = (runtime_measured.as_ticks() - 1 * TICKS_PER_US) / period.as_ticks()
                    + if PRE_TRIGGER.is_some() { 1 } else { 0 };
                if *count != expected {
                    sprintln!("{TAG_WARN} {id} has {count} completions, expected {expected}",);
                }
                if *spurious_count != 0 {
                    sprintln!(
                        "{TAG_WARN} {id} has suffered {spurious_count} spurious interrupts. Last completed at: {} us",
                        last_spurious.as_micros()
                    );
                }
            }

            for TaskStat {
                id,
                prio,
                worst,
                miss_count,
                count,
                ..
            } in stats
            {
                let worst = worst.as_micros();
                sprintln!(
                    "- {id:>10} (p={prio:#x}): worst {worst:>5} us | n_complete {count:>4} | misses {miss_count:>4}"
                );
            }
        }

        fn filter_spurious(&mut self) -> bool {
            let now = MTimerLo::instance().now();

            // Theoretical arrival time of the job being completed
            //
            // `SYS_START` occurs right after setting off the periodic timers.
            let next_arrival = unsafe { SYS_START }
                + (self.count + if PRE_TRIGGER.is_some() { 0 } else { 1 }) * self.period;

            // The periodic timer never fires before its period is completed, so
            // any execution before the next scheduled arrival cannot belong to
            // a new job; it is a spurious re-dispatch of the current job. Drop
            // it instead of counting it as a new job. A genuine job always
            // executes after its own arrival, as its work duration is non-zero.
            //
            // The issue is currently traced to the CLIC, which re-presents a
            // source whose `ip` was not cleared during a simultaneous
            // multi-source collision.
            (now < next_arrival)
                .then(|| {
                    self.spurious_count += 1;
                    self.last_spurious = now;
                })
                .is_some()
        }

        /// Record that the job has completed
        fn report_job_complete(&mut self) {
            // Record time of job completion as early as possible
            let t_complete = MTimerLo::instance().now();

            // Theoretical arrival time of the job being completed
            //
            // `SYS_START` occurs right after setting off the periodic timers.
            let next_arrival = unsafe { SYS_START }
                + (self.count + if PRE_TRIGGER.is_some() { 0 } else { 1 }) * self.period;

            // Record completion
            self.count += 1;

            // If completed late, record deadline miss
            if t_complete > next_arrival + self.dl {
                self.miss_count += 1;
            }
            // If completed on time, record worst response time
            else {
                // `t_resp` may slightly underreport, as arrival times
                // are based on calculated periods.
                let t_resp = t_complete.checked_sub(next_arrival).unwrap_or_else(|| {
                    panic!(
                        "[ERROR] {}: job arrived after it was completed {next_arrival} > {t_complete}",
                        self.id
                    )
                });
                if t_resp > self.worst {
                    self.worst = t_resp;
                }
            }
        }
    }
    static mut STAT_R_HI: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_J: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_R_LO: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_W: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut SYS_START: Duration32 = Duration32::ZERO;

    #[inline]
    fn wait_ticks(t: u32) {
        // One loop takes 6 cycles (3 instructions); verified from execution trace:
        //
        // ```
        // addi	x0,   0
        // addi	x10, -1
        // bnez	x10, {}
        // ```
        for _ in 0..t / 6 {
            unsafe { core::arch::asm!("nop") }
        }
    }

    #[shared]
    struct Shared {
        /// The protected readers-writer resource
        r: u32,
    }

    #[init]
    fn init() -> Shared {
        ApbUart::init(CPU_FREQ_HZ, 115_200);

        let mtimer_res_ns = Duration32::from_ticks(1).as_nanos();
        let mtimer_max_ms = Duration32::MAX.as_millis();

        // Ensure no critical section undercuts mtimer resolution, which is used
        // as the await delay.
        assert!(CS_RHI.as_nanos() >= mtimer_res_ns);
        assert!(WORK_J.as_nanos() >= mtimer_res_ns);
        assert!(CS_RLO.as_nanos() >= mtimer_res_ns);
        assert!(CS_W.as_nanos() >= mtimer_res_ns);

        sprintln!("\r\n### RW-lock schedulability demo (zeroHETI / RTIC) ###");
        sprintln!("- Timer res.   (ns) : {mtimer_res_ns:?}",);
        sprintln!("- Timer max.   (ms) : {mtimer_max_ms:?}",);
        sprintln!("- Lock mode         : {LOCK_MODE}");
        sprintln!("- RUNTIME_MS        : {RUNTIME_MS}");
        sprintln!("Task set:");
        sprintln!("- Hyperperiod  (ms) : {}", HYPERPERIOD_US / 1_000);
        sprintln!("- Theoretical load  : {UTIL_MIN_PC}%");
        sprintln!("- ReaderLow CS (us) : {}", CS_RLO.as_micros());
        sprintln!("- J deadline   (us) : {}", DL_J_US);

        sprintln!("Control::Setup");

        // Setup mtimer to trigger the `Finish` task
        MTimerLo::instance().start(Duration32::from_millis(RUNTIME_MS));

        let timers: &mut [bsp::timer_group::Periodic; 4] = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
            Timer::init::<TIMER3_ADDR>().into_periodic(),
        ];
        if let Some(initial_delay) = PRE_TRIGGER {
            timers[0].set_period_next(PERIOD_RHI_US.micros(), initial_delay);
            timers[1].set_period_next(PERIOD_J_US.micros(), initial_delay);
            timers[2].set_period_next(PERIOD_RLO_US.micros(), initial_delay);
            timers[3].set_period_next(PERIOD_W_US.micros(), initial_delay);
        } else {
            timers[0].set_period(PERIOD_RHI_US.micros());
            timers[1].set_period(PERIOD_J_US.micros());
            timers[2].set_period(PERIOD_RLO_US.micros());
            timers[3].set_period(PERIOD_W_US.micros());
        }
        timers.iter_mut().for_each(|t| t.start());

        clear_perf_counters();
        unsafe { SYS_START = MTimerLo::instance().now() };

        Shared { r: 42 }
    }

    #[task(
        binds = MachineTimer,
        priority = 0xff,
    )]
    struct Teardown {}
    impl RticTask for Teardown {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let runtime_measured = MTimerLo::instance().now() - unsafe { SYS_START };
            let minstret = minstret::read64();
            let mcycle = mcycle::read64();

            sprintln!("Control::Teardown");
            sprintln!("- Runtime      (us) : {}", runtime_measured.as_micros());
            let cpu_util_pc = (mcycle * 100) / runtime_measured.as_ticks() as u64;
            sprintln!(
                "- True CPU util.    : {}%, instr. count: {}",
                cpu_util_pc,
                minstret
            );
            if cpu_util_pc < UTIL_MIN_PC as u64 {
                sprintln!("[WARN] CPU util less than minimum. Less than expected jobs were run.")
            }

            // Print statistics for all tasks
            unsafe {
                TaskStat::print_stats(
                    &runtime_measured,
                    &[
                        &STAT_R_HI.assume_init_read(),
                        &STAT_J.assume_init_read(),
                        &STAT_R_LO.assume_init_read(),
                        &STAT_W.assume_init_read(),
                    ],
                )
            };

            // Report job J, which is of special interest
            let TaskStat {
                dl,
                miss_count,
                count,
                ..
            } = unsafe { STAT_J.assume_init_read() };
            sprintln!(
                "VERDICT [{LOCK_MODE}]: J{} schedulable -- {miss_count}/{count} jobs missed the {} us deadline",
                if miss_count > 0 { " NOT" } else { "" },
                dl.as_micros()
            );

            #[cfg(feature = "obs")]
            obs_trace::obs_dump!(obs_trace::TsUnit::Micros);

            // HACK: wait for prints to complete
            MTimerLo::instance().wait_busy(Duration32::from_millis(1));
            signal_pass(None);
        }
    }

    /// ReaderHigh: high-priority reader, short critical section.
    /// Accesses R in read mode (does not block J) in the RW build.
    #[cfg(feature = "rw")]
    #[task(binds = Timer0Cmp, priority = 0xfc, read = [r])]
    struct ReaderHigh {
        cs_duration: Duration32,
    }
    #[cfg(not(feature = "rw"))]
    #[task(binds = Timer0Cmp, priority = 0xfc, shared = [r])]
    struct ReaderHigh {
        cs_duration: Duration32,
    }
    impl RticTask for ReaderHigh {
        fn init() -> Self {
            unsafe {
                STAT_R_HI.write(TaskStat::new(
                    "ReaderHigh",
                    0xfc,
                    Duration32::from_micros(PERIOD_RHI_US),
                    Duration32::from_micros(PERIOD_RHI_US),
                ))
            };
            Self {
                cs_duration: CS_RHI,
            }
        }
        fn exec(&mut self) {
            // Spurious job detection:
            //
            // If a job is executing before its arrival time, it is a
            // spurious job spawned by the known hardware bug.
            //
            // Filter out it out as early as possible.
            unsafe { STAT_R_HI.assume_init_read() }.filter_spurious();

            #[cfg(feature = "rw")]
            self.shared()
                .r
                .read_lock(|_r| wait_ticks(self.cs_duration.as_ticks()));
            #[cfg(not(feature = "rw"))]
            self.shared()
                .r
                .lock(|_r| wait_ticks(self.cs_duration.as_ticks()));

            unsafe { STAT_R_HI.assume_init_mut() }.report_job_complete();
        }
    }

    /// J: high-priority task that does NOT access R. Its response time is the
    /// schedulability witness: it must preempt reader critical sections.
    #[task(binds = Timer1Cmp, priority = 0xfb, shared = [])]
    struct J {
        work_duration: Duration32,
    }
    impl RticTask for J {
        fn init() -> Self {
            unsafe {
                STAT_J.write(TaskStat::new(
                    "J",
                    0xfb,
                    Duration32::from_micros(PERIOD_J_US),
                    Duration32::from_micros(DL_J_US),
                ))
            };
            Self {
                work_duration: WORK_J,
            }
        }
        fn exec(&mut self) {
            // Spurious job detection:
            //
            // If a job is executing before its arrival time, it is a
            // spurious job spawned by the known hardware bug.
            //
            // Filter out it out as early as possible.
            unsafe { STAT_J.assume_init_read() }.filter_spurious();

            wait_ticks(self.work_duration.as_ticks());

            unsafe { STAT_J.assume_init_mut() }.report_job_complete();
        }
    }

    /// ReaderLow: low-priority reader with a LONG critical section.
    /// Its read CS is the dominant term of J's mutex-mode response time.
    #[cfg(feature = "rw")]
    #[task(binds = Timer2Cmp, priority = 0xf9, shared = [], read = [r])]
    struct ReaderLow {
        cs_duration: Duration32,
    }
    #[cfg(not(feature = "rw"))]
    #[task(binds = Timer2Cmp, priority = 0xf9, shared = [r])]
    struct ReaderLow {
        cs_duration: Duration32,
    }
    impl RticTask for ReaderLow {
        fn init() -> Self {
            unsafe {
                STAT_R_LO.write(TaskStat::new(
                    "ReaderLow",
                    0xf9,
                    Duration32::from_micros(PERIOD_RLO_US),
                    Duration32::from_micros(PERIOD_RLO_US),
                ))
            };
            Self {
                cs_duration: CS_RLO,
            }
        }
        fn exec(&mut self) {
            // Spurious job detection:
            //
            // If a job is executing before its arrival time, it is a
            // spurious job spawned by the known hardware bug.
            //
            // Filter out it out as early as possible.
            unsafe { STAT_R_LO.assume_init_read() }.filter_spurious();

            #[cfg(feature = "rw")]
            self.shared()
                .r
                .read_lock(|_r| wait_ticks(self.cs_duration.as_ticks()));
            #[cfg(not(feature = "rw"))]
            self.shared()
                .r
                .lock(|_r| wait_ticks(self.cs_duration.as_ticks()));

            unsafe { STAT_R_LO.assume_init_mut() }.report_job_complete();
        }
    }

    /// Writer: the only writer of R (sets the RW read-lock ceiling).
    #[task(binds = Timer3Cmp, priority = 0xf8, shared = [r])]
    struct Writer {
        cs_duration: Duration32,
    }
    impl RticTask for Writer {
        fn init() -> Self {
            unsafe {
                STAT_W.write(TaskStat::new(
                    "Writer",
                    0xf8,
                    Duration32::from_micros(PERIOD_W_US),
                    Duration32::from_micros(PERIOD_W_US),
                ))
            };
            Self { cs_duration: CS_W }
        }
        fn exec(&mut self) {
            // Spurious job detection:
            //
            // If a job is executing before its arrival time, it is a
            // spurious job spawned by the known hardware bug.
            //
            // Filter out it out as early as possible.
            unsafe { STAT_W.assume_init_read() }.filter_spurious();

            self.shared()
                .r
                .lock(|_r| wait_ticks(self.cs_duration.as_ticks()));

            unsafe { STAT_W.assume_init_mut() }.report_job_complete();
        }
    }
}
