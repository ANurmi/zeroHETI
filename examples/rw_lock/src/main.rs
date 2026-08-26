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
        fugit::{ExtU32, ExtU64},
        mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mtimer::{Duration32, MTimer},
        parse_u32, sprintln,
        tb::signal_pass,
        timer_group::Timer,
    };
    use core::mem::MaybeUninit;

    // # Hyperparams

    /// Runtime in milliseconds, read from env RUNTIME_MS (compile-time)
    const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;
    const LOCK_MODE: &str = if cfg!(feature = "rw") {
        "rw-lock"
    } else {
        "mutex"
    };

    /// Period of `ReaderHigh`
    const PERIOD_RHI_US: u32 = 700;
    /// Period of `J`
    const PERIOD_J_US: u32 = 1300;
    /// Period of `ReaderLow`
    const PERIOD_RLO_US: u32 = 1000;
    /// Period of `W`
    const PERIOD_W_US: u32 = 1500;

    /// Duration of the critical section of `ReaderHigh`. Short.
    const CS_RHI: Duration32 = Duration32::from_micros(15);
    /// Duration of the work of `J`.
    const WORK_J: Duration32 = Duration32::from_micros(350);
    /// Duration of the critical section of `ReaderLow`. Long.
    const CS_RLO: Duration32 = Duration32::from_micros(700);
    /// Duration of the critical section of `W`. Short.
    const CS_W: Duration32 = Duration32::from_micros(16);

    const DL_J_US: u32 = 400;

    // # Global records

    /// Per task job statistics
    #[derive(Clone, Copy)]
    pub struct TaskStat {
        /// Worst observed response time
        worst: Duration32,
        /// Number of completed jobs
        count: u32,
    }
    impl core::default::Default for TaskStat {
        fn default() -> TaskStat {
            TaskStat {
                worst: Duration32::ZERO,
                count: 0,
            }
        }
    }
    impl TaskStat {
        /// Record that the job has completed
        ///
        /// # Arguments
        ///
        /// * `t_resp` - response time
        fn report_job_complete(&mut self, t_resp: Duration32) {
            self.count += 1;
            if t_resp > self.worst {
                self.worst = t_resp;
            }
        }
    }
    static mut STAT_R_HI: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_J: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_R_LO: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut STAT_W: MaybeUninit<TaskStat> = MaybeUninit::uninit();
    static mut J_MISSES: usize = 0;
    static mut SYS_START: Duration32 = Duration32::ZERO;

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
        sprintln!("- RL CS        (us) : {}", CS_RLO.as_micros());
        sprintln!("- J deadline   (us) : {}", DL_J_US);

        sprintln!("Control::Setup");

        // Setup mtimer to trigger `Finish` task
        let mut mtimer = MTimer::instance().into_oneshot();
        mtimer.start(RUNTIME_MS.millis());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
            Timer::init::<TIMER3_ADDR>().into_periodic(),
        ];
        timers[0].set_period(PERIOD_RHI_US.micros());
        timers[1].set_period(PERIOD_J_US.micros());
        timers[2].set_period(PERIOD_RLO_US.micros());
        timers[3].set_period(PERIOD_W_US.micros());
        timers.iter_mut().for_each(|t| t.start());

        unsafe { SYS_START = MTimer::instance().into_lo().now() };

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
            sprintln!("Control::Teardown");
            sprintln!(
                "- Runtime (us): {}",
                (MTimer::instance().now() - unsafe { SYS_START }).as_micros()
            );

            struct AllStats {
                r_hi: TaskStat,
                j: TaskStat,
                r_lo: TaskStat,
                w: TaskStat,
            }
            let stat = unsafe {
                AllStats {
                    r_hi: STAT_R_HI.assume_init_read(),
                    j: STAT_J.assume_init_read(),
                    r_lo: STAT_R_LO.assume_init_read(),
                    w: STAT_W.assume_init_read(),
                }
            };

            let (rh_w, rh_c) = (stat.r_hi.worst.as_micros(), stat.r_hi.count);
            let (j_w, j_c, j_m) = (stat.j.worst.as_micros(), stat.j.count, unsafe { J_MISSES });
            let (rl_w, rl_c) = (stat.r_lo.worst.as_micros(), stat.r_lo.count);
            let (w_w, w_c) = (stat.w.worst.as_micros(), stat.w.count);

            sprintln!("- ReaderHigh (p=0xfc): worst {rh_w:>5} us | n_complete {rh_c:>4}");
            sprintln!(
                "- J          (p=0xfb): worst {j_w:>5} us | n_complete {j_c:>4} | misses {j_m:>4}"
            );
            sprintln!("- ReaderLow  (p=0xf9): worst {rl_w:>5} us | n_complete {rl_c:>4}");
            sprintln!("- Writer     (p=0xf8): worst {w_w:>5} us | n_complete {w_c:>4}");

            if j_m > 0 {
                sprintln!(
                    "VERDICT [{LOCK_MODE}]: J NOT schedulable -- {j_m}/{j_c} jobs missed the {DL_J_US} us deadline",
                );
            } else {
                sprintln!(
                    "VERDICT [{LOCK_MODE}]: J schedulable -- 0/{j_c} jobs missed the {DL_J_US} us deadline"
                );
            }

            #[cfg(feature = "obs")]
            obs_trace::obs_dump!(obs_trace::TsUnit::Micros);

            // HACK: wait for prints to complete
            MTimer::instance()
                .into_lo()
                .wait_busy(Duration32::from_millis(1));
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
            unsafe { STAT_R_HI.write(TaskStat::default()) };
            Self {
                cs_duration: CS_RHI,
            }
        }
        fn exec(&mut self) {
            let timer = unsafe { Timer::instance::<TIMER0_ADDR>() };
            let mtimer = MTimer::instance().into_lo();

            #[cfg(feature = "rw")]
            self.shared()
                .r
                .read_lock(|_r| mtimer.wait_busy(self.cs_duration));
            #[cfg(not(feature = "rw"))]
            self.shared()
                .r
                .lock(|_r| mtimer.wait_busy(self.cs_duration));

            // APB Timer reset coincides with task release time =>
            // timer.duration() returns response time
            let t_resp = timer.duration();
            unsafe { STAT_R_HI.assume_init_mut() }.report_job_complete(t_resp);
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
            unsafe { STAT_J.write(TaskStat::default()) };
            Self {
                work_duration: WORK_J,
            }
        }
        fn exec(&mut self) {
            let timer = unsafe { Timer::instance::<TIMER1_ADDR>() };
            let mtimer = MTimer::instance().into_lo();

            mtimer.wait_busy(self.work_duration);

            // APB Timer reset coincides with task release time =>
            // timer.duration() returns response time
            let t_resp = timer.duration();
            unsafe { STAT_J.assume_init_mut() }.report_job_complete(t_resp);
            if t_resp > Duration32::from_micros(DL_J_US) {
                unsafe { J_MISSES += 1 };
            }
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
            unsafe { STAT_R_LO.write(TaskStat::default()) };
            Self {
                cs_duration: CS_RLO,
            }
        }
        fn exec(&mut self) {
            let timer = unsafe { Timer::instance::<TIMER2_ADDR>() };
            let mtimer = MTimer::instance().into_lo();

            #[cfg(feature = "rw")]
            self.shared()
                .r
                .read_lock(|_r| mtimer.wait_busy(self.cs_duration));
            #[cfg(not(feature = "rw"))]
            self.shared()
                .r
                .lock(|_r| mtimer.wait_busy(self.cs_duration));

            // APB Timer reset coincides with task release time =>
            // timer.duration() returns response time
            let t_resp = timer.duration();
            unsafe { STAT_R_LO.assume_init_mut() }.report_job_complete(t_resp);
        }
    }

    /// Writer: the only writer of R (sets the RW read-lock ceiling).
    #[task(binds = Timer3Cmp, priority = 0xf8, shared = [r])]
    struct Writer {
        cs_duration: Duration32,
    }
    impl RticTask for Writer {
        fn init() -> Self {
            unsafe { STAT_W.write(TaskStat::default()) };
            Self { cs_duration: CS_W }
        }
        fn exec(&mut self) {
            let timer = unsafe { Timer::instance::<TIMER3_ADDR>() };
            let mtimer = MTimer::instance().into_lo();

            self.shared()
                .r
                .lock(|_r| mtimer.wait_busy(self.cs_duration));

            // APB Timer reset coincides with task release time =>
            // timer.duration() returns response time
            let t_resp = timer.duration();
            unsafe { STAT_W.assume_init_mut() }.report_job_complete(t_resp);
        }
    }
}
