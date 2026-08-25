#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;

#[cfg_attr(feature = "obs", rtic::app(device = bsp, obs = obs_trace::Obs, dispatchers = []))]
#[cfg_attr(not(feature = "obs"), rtic::app(device = bsp, dispatchers = []))]
mod app {
    const fn parse_u32(s: &str) -> u32 {
        let mut out: u32 = 0;
        let mut i: usize = 0;
        while i < s.len() {
            out *= 10;
            out += (s.as_bytes()[i] - b'0') as u32;
            i += 1;
        }
        out
    }

    /// Runtime in milliseconds, read from env RUNTIME_MS (compile-time)
    const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;
    /// Length of the ReaderLow critical section, in busy-loop iterations
    const RL_CS_ITERS: u32 = match option_env!("RL_CS_ITERS") {
        Some(s) => parse_u32(s),
        None => 900,
    };
    /// Deadline of the measured (non-accessing) task J, in microseconds
    const DL_J_US: u32 = match option_env!("DL_J_US") {
        Some(s) => parse_u32(s),
        None => 400,
    };

    // Periods of the periodic tasks (us)
    const PER_RH_US: u32 = 700;
    const PER_J_US: u32 = 1300;
    const PER_RL_US: u32 = 1000;
    const PER_W_US: u32 = 1500;

    #[cfg(feature = "rw")]
    const LOCK_MODE: &str = "rw-lock";
    #[cfg(not(feature = "rw"))]
    const LOCK_MODE: &str = "mutex";

    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        fugit::{ExtU32, ExtU64},
        mmap::apb_timer::{TIMER_SEP, TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mtimer::MTimer,
        register::{mcycle, mcycleh, minstret, minstreth},
        sprintln,
        tb::signal_pass,
        timer_group::Timer,
    };

    /// Number of timer ticks per microsecond (prescaler 0, 1 tick == 1 CPU cycle)
    const HZ_PER_US: u32 = CPU_FREQ_HZ / 1_000_000;

    const PER_J_TICKS: u32 = PER_J_US * HZ_PER_US;
    const DL_J_TICKS: u32 = DL_J_US * HZ_PER_US;

    /// The protected resource R: an 8-word state buffer.
    const R_LEN: usize = 8;

    #[derive(Clone, Copy)]
    pub struct RState {
        data: [u32; R_LEN],
        wgen: u32,
    }

    #[derive(Clone, Copy, Default)]
    pub struct TaskStat {
        /// Worst observed response time, in timer ticks
        worst: u32,
        /// Number of executed jobs
        count: u32,
        /// Number of jobs exceeding the deadline (J only)
        misses: u32,
    }

    #[derive(Clone, Copy)]
    pub struct Stats {
        rh: TaskStat,
        j: TaskStat,
        rl: TaskStat,
        w: TaskStat,
    }

    #[shared]
    struct Shared {
        /// The protected readers-writer resource
        r: RState,
        /// Per-task worst-case response-time statistics
        stats: Stats,
    }

    #[inline]
    fn timer_counter(idx: usize) -> u32 {
        unsafe { Timer::instance_dyn(TIMER0_ADDR + idx * TIMER_SEP) }.counter()
    }

    /// Response time from the APB timer counter value, with single-wrap handling
    #[inline]
    fn wrap(resp: u32, start: u32, period: u32) -> u32 {
        if resp < start { resp + period } else { resp }
    }

    #[inline]
    fn track(st: &mut TaskStat, resp: u32) {
        st.count += 1;
        if resp > st.worst {
            st.worst = resp;
        }
    }

    #[inline]
    fn clear_perf_counters() {
        unsafe {
            minstret::write(0);
            mcycle::write(0);
            minstreth::write(0);
            mcycleh::write(0);
        }
    }

    /// Per-job record for the visualization: response time (APB ticks) and
    /// absolute end-of-job timestamp (mtimer ticks).
    #[derive(Clone, Copy)]
    struct JobRec {
        resp: u32,
        end: u64,
    }

    /// Per-task job logs (each written by exactly one task, read at teardown).
    const LOG_CAP: usize = 64;
    static mut RH_LOG: [JobRec; LOG_CAP] = [JobRec { resp: 0, end: 0 }; LOG_CAP];
    static mut J_LOG: [JobRec; LOG_CAP] = [JobRec { resp: 0, end: 0 }; LOG_CAP];
    static mut RL_LOG: [JobRec; LOG_CAP] = [JobRec { resp: 0, end: 0 }; LOG_CAP];
    static mut W_LOG: [JobRec; LOG_CAP] = [JobRec { resp: 0, end: 0 }; LOG_CAP];
    static mut RH_LOG_LEN: usize = 0;
    static mut J_LOG_LEN: usize = 0;
    static mut RL_LOG_LEN: usize = 0;
    static mut W_LOG_LEN: usize = 0;

    #[inline]
    fn log_job(buf: &mut [JobRec; LOG_CAP], len: &mut usize, resp: u32) {
        let i = *len;
        if i < LOG_CAP {
            buf[i] = JobRec {
                resp,
                end: MTimer::instance().counter(),
            };
            *len = i + 1;
        }
    }

    /// Prints one `JOB <task> <idx> <resp_us> <rel_us> <miss>` line per recorded
    /// job, for the visualization. `rel_us` is the release time relative to the
    /// run start, derived from the absolute mtimer timestamp and the response.
    ///
    /// # Safety
    ///
    /// Reads shared variables unsafely. This can only be called safely when
    /// there's no concurrency in app (at end of program).
    #[inline(never)]
    unsafe fn dump_logs(start_ticks: u64) {
        sprintln!("MODE {LOCK_MODE}");
        let start_us = start_ticks / (HZ_PER_US as u64);
        unsafe {
            macro_rules! dump {
                ($tag:expr, $log:expr, $len:expr, $dl:expr) => {{
                    for i in 0..*$len {
                        let rec = &$log[i];
                        let rel_us = (rec.end - rec.resp as u64) / (HZ_PER_US as u64) - start_us;
                        let resp_us = rec.resp / HZ_PER_US;
                        let miss = if $dl > 0 && rec.resp > $dl { 1 } else { 0 };
                        sprintln!("JOB {} {} {} {} {}", $tag, i, resp_us, rel_us, miss);
                    }
                }};
            }
            dump!("RH", &RH_LOG, &RH_LOG_LEN, 0);
            dump!("J", &J_LOG, &J_LOG_LEN, DL_J_TICKS);
            dump!("RL", &RL_LOG, &RL_LOG_LEN, 0);
            dump!("W", &W_LOG, &W_LOG_LEN, 0);
        }
    }

    /// Long read critical section: touches `r` once, then folds a synthetic
    /// accumulation into `acc`. `iters` controls the critical-section length.
    #[inline(never)]
    fn read_busy(r: &RState, seed: u32, iters: u32) -> u32 {
        let mut acc = seed ^ r.data[0].wrapping_add(r.wgen);
        let mut i = 0u32;
        while i < iters {
            acc = acc.wrapping_mul(3).wrapping_add(0x9e37_79b9);
            i += 1;
        }
        acc
    }

    /// Short read critical section (ReaderHigh)
    #[inline(never)]
    fn read_short(r: &RState, seed: u32) -> u32 {
        let mut acc = seed;
        for x in r.data.iter() {
            acc = acc.wrapping_add(*x);
        }
        acc.wrapping_add(r.wgen)
    }

    /// Benign computation of the non-accessing task J (does not touch R)
    #[inline(never)]
    fn j_work(seed: u32) -> u32 {
        let mut acc = seed;
        for i in 0..32 {
            acc = acc.wrapping_mul(3).wrapping_add(i);
        }
        acc
    }

    #[init]
    fn init() -> Shared {
        ApbUart::init(CPU_FREQ_HZ, 115_200);
        sprintln!("\r\n### RW-lock schedulability demo (zeroHETI / RTIC) ###");
        sprintln!("- Lock mode         : {LOCK_MODE}");
        sprintln!("- RUNTIME_MS        : {RUNTIME_MS}");
        sprintln!("- RL CS iters       : {RL_CS_ITERS}");
        sprintln!("- J deadline   (us) : {DL_J_US}");

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
        timers[0].set_period(PER_RH_US.micros());
        timers[1].set_period(PER_J_US.micros());
        timers[2].set_period(PER_RL_US.micros());
        timers[3].set_period(PER_W_US.micros());
        timers.iter_mut().for_each(|t| t.start());

        clear_perf_counters();

        Shared {
            r: RState {
                data: [1, 2, 3, 4, 5, 6, 7, 8],
                wgen: 7,
            },
            stats: Stats {
                rh: TaskStat::default(),
                j: TaskStat::default(),
                rl: TaskStat::default(),
                w: TaskStat::default(),
            },
        }
    }

    #[task(
        binds = MachineTimer,
        priority = 0xff,
        shared = [stats]
    )]
    struct Finish {}

    impl RticTask for Finish {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let now = MTimer::instance().into_oneshot().duration().as_ticks();
            let runtime_us = now / (HZ_PER_US as u64);

            sprintln!("Control::Teardown");
            sprintln!("- Runtime (us): {runtime_us}");

            self.shared().stats.lock(|st| {
                let (rh_w, rh_c) = (st.rh.worst / HZ_PER_US, st.rh.count);
                let (j_w, j_c, j_m) = (
                    st.j.worst / HZ_PER_US,
                    st.j.count,
                    st.j.misses,
                );
                let (rl_w, rl_c) = (st.rl.worst / HZ_PER_US, st.rl.count);
                let (w_w, w_c) = (st.w.worst / HZ_PER_US, st.w.count);

                sprintln!("- ReaderHigh (0xfc): worst {rh_w:>5} us | jobs {rh_c:>4}");
                sprintln!("- J         (0xfb): worst {j_w:>5} us | jobs {j_c:>4} | misses {j_m:>4}");
                sprintln!("- ReaderLow (0xf9): worst {rl_w:>5} us | jobs {rl_c:>4}");
                sprintln!("- Writer    (0xf8): worst {w_w:>5} us | jobs {w_c:>4}");

                if j_m > 0 {
                    sprintln!("VERDICT [{LOCK_MODE}]: J NOT schedulable -- {j_m}/{j_c} jobs missed the {DL_J_US} us deadline");
                } else {
                    sprintln!("VERDICT [{LOCK_MODE}]: J schedulable -- 0/{j_c} jobs missed the {DL_J_US} us deadline");
                }
            });

            unsafe { dump_logs(0) };
            #[cfg(feature = "obs")]
            obs_trace::obs_dump!(obs_trace::TsUnit::Micros);

            signal_pass(None);
        }
    }

    /// ReaderHigh: high-priority reader, short critical section.
    /// Accesses R in read mode (does not block J) in the RW build.
    #[cfg(feature = "rw")]
    #[task(binds = Timer0Cmp, priority = 0xfc, shared = [stats], read = [r])]
    struct ReaderHigh {
        acc: u32,
    }
    #[cfg(not(feature = "rw"))]
    #[task(binds = Timer0Cmp, priority = 0xfc, shared = [stats, r])]
    struct ReaderHigh {
        acc: u32,
    }
    #[cfg(feature = "rw")]
    impl RticTask for ReaderHigh {
        fn init() -> Self {
            Self { acc: 0 }
        }
        fn exec(&mut self) {
            let start = timer_counter(0);
            self.shared()
                .r
                .read_lock(|r| self.acc = read_short(r, self.acc));
            let resp = wrap(timer_counter(0), start, PER_RH_US * HZ_PER_US);
            unsafe { log_job(&mut RH_LOG, &mut RH_LOG_LEN, resp) };
            self.shared().stats.lock(|st| track(&mut st.rh, resp));
        }
    }
    #[cfg(not(feature = "rw"))]
    impl RticTask for ReaderHigh {
        fn init() -> Self {
            Self { acc: 0 }
        }
        fn exec(&mut self) {
            let start = timer_counter(0);
            self.shared().r.lock(|r| self.acc = read_short(r, self.acc));
            let resp = wrap(timer_counter(0), start, PER_RH_US * HZ_PER_US);
            unsafe { log_job(&mut RH_LOG, &mut RH_LOG_LEN, resp) };
            self.shared().stats.lock(|st| track(&mut st.rh, resp));
        }
    }

    /// J: high-priority task that does NOT access R. Its response time is the
    /// schedulability witness: it must preempt reader critical sections.
    #[task(binds = Timer1Cmp, priority = 0xfb, shared = [stats])]
    struct J {
        acc: u32,
    }
    impl RticTask for J {
        fn init() -> Self {
            Self { acc: 0 }
        }
        fn exec(&mut self) {
            let start = timer_counter(1);
            self.acc = j_work(self.acc);
            let resp = wrap(timer_counter(1), start, PER_J_TICKS);
            unsafe { log_job(&mut J_LOG, &mut J_LOG_LEN, resp) };
            self.shared().stats.lock(|st| {
                st.j.count += 1;
                if resp > st.j.worst {
                    st.j.worst = resp;
                }
                if resp > DL_J_TICKS {
                    st.j.misses += 1;
                }
            });
        }
    }

    /// ReaderLow: low-priority reader with a LONG critical section.
    /// Its read CS is the dominant term of J's mutex-mode response time.
    #[cfg(feature = "rw")]
    #[task(binds = Timer2Cmp, priority = 0xf9, shared = [stats], read = [r])]
    struct ReaderLow {
        acc: u32,
    }
    #[cfg(not(feature = "rw"))]
    #[task(binds = Timer2Cmp, priority = 0xf9, shared = [stats, r])]
    struct ReaderLow {
        acc: u32,
    }
    #[cfg(feature = "rw")]
    impl RticTask for ReaderLow {
        fn init() -> Self {
            Self { acc: 0 }
        }
        fn exec(&mut self) {
            let start = timer_counter(2);
            self.shared()
                .r
                .read_lock(|r| self.acc = read_busy(r, self.acc, RL_CS_ITERS));
            let resp = wrap(timer_counter(2), start, PER_RL_US * HZ_PER_US);
            unsafe { log_job(&mut RL_LOG, &mut RL_LOG_LEN, resp) };
            self.shared().stats.lock(|st| track(&mut st.rl, resp));
        }
    }
    #[cfg(not(feature = "rw"))]
    impl RticTask for ReaderLow {
        fn init() -> Self {
            Self { acc: 0 }
        }
        fn exec(&mut self) {
            let start = timer_counter(2);
            self.shared()
                .r
                .lock(|r| self.acc = read_busy(r, self.acc, RL_CS_ITERS));
            let resp = wrap(timer_counter(2), start, PER_RL_US * HZ_PER_US);
            unsafe { log_job(&mut RL_LOG, &mut RL_LOG_LEN, resp) };
            self.shared().stats.lock(|st| track(&mut st.rl, resp));
        }
    }

    /// Writer: the only writer of R (sets the RW read-lock ceiling).
    #[task(binds = Timer3Cmp, priority = 0xf8, shared = [stats, r])]
    struct Writer {}
    impl RticTask for Writer {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let start = timer_counter(3);
            self.shared().r.lock(|r| {
                for x in r.data.iter_mut() {
                    *x = x.wrapping_mul(0x10203).wrapping_add(0x1100_22);
                }
                r.wgen = r.wgen.wrapping_add(1);
            });
            let resp = wrap(timer_counter(3), start, PER_W_US * HZ_PER_US);
            unsafe { log_job(&mut W_LOG, &mut W_LOG_LEN, resp) };
            self.shared().stats.lock(|st| track(&mut st.w, resp));
        }
    }
}
