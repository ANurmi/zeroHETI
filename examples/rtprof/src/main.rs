#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;

#[cfg_attr(feature = "obs", rtic::app(device = bsp, obs = obs_trace::Obs /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
#[cfg_attr(not(feature = "obs"), rtic::app(device = bsp /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
mod app {
    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        cfg_regs::CfgRegs,
        clear_perf_counters,
        fugit::{ExtU32, ExtU64},
        i2c::{self, I2c},
        lcm,
        mailbox::Mailbox,
        mmap::{
            apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR},
            cfg_regs::CFG_BASE_ADDR,
        },
        mmio,
        mtimer::*,
        parse_u32,
        register::{mcycle, minstret},
        sprintln,
        tb::signal_pass,
        timer_group::{Periodic, Timer},
    };

    use libm::sin;

    const CFG_TASK_OFFS: usize = 0x0000_0100;

    #[inline]
    fn rtprof_start_task(idx: usize) {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + (4 + 4 * idx), 1);
    }

    #[inline]
    fn rtprof_end_task(idx: usize) {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + (4 + 4 * idx), 0);
    }

    #[inline]
    fn rtprof_start_full() {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS, 2);
    }

    #[inline]
    fn rtprof_stop() {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS, 0);
    }

    const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
    const RT: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

    struct Task {
        period_us: u32,
        deadline_us: u32,
        runtime_us: u32,
    }

    impl Task {
        pub const fn new(period: u32, deadline: u32, runtime: u32) -> Self {
            assert!(
                deadline >= runtime,
                "Deadline cannot be shorter than unblocked runtime"
            );
            assert!(period >= deadline, "Period cannot be shorter than deadline");
            assert!(
                255 >= deadline,
                "8-bits and 1 us tick limit deadlines to 0..255 us"
            );
            Self {
                period_us: (period),
                deadline_us: (deadline),
                runtime_us: (runtime),
            }
        }
    }

    const TASK_SET_SIZE: usize = 5;

    const TASK_SET: [Task; TASK_SET_SIZE] = [
        Task::new(130, 30, 8 * LF / 100),
        Task::new(66, 50, 30 * LF / 100),
        Task::new(270, 150, 50 * LF / 100),
        Task::new(200, 150, 50 * LF / 100),
        Task::new(70, 20, 1 * LF / 100),
    ];

    const HYPERPERIOD: u32 = lcm!(
        TASK_SET[0].period_us,
        TASK_SET[1].period_us,
        TASK_SET[2].period_us,
        TASK_SET[3].period_us,
        TASK_SET[4].period_us,
    );

    // Frequency within hyperperiod
    const TASK_FREQ: [u32; TASK_SET_SIZE] = [
        HYPERPERIOD / TASK_SET[0].period_us,
        HYPERPERIOD / TASK_SET[1].period_us,
        HYPERPERIOD / TASK_SET[2].period_us,
        HYPERPERIOD / TASK_SET[3].period_us,
        HYPERPERIOD / TASK_SET[4].period_us,
    ];

    const TASK_RT: [u32; TASK_SET_SIZE] = [
        TASK_FREQ[0] * TASK_SET[0].runtime_us,
        TASK_FREQ[1] * TASK_SET[1].runtime_us,
        TASK_FREQ[2] * TASK_SET[2].runtime_us,
        TASK_FREQ[3] * TASK_SET[3].runtime_us,
        TASK_FREQ[4] * TASK_SET[4].runtime_us,
    ];

    const TASK_RT_TOT: u32 = TASK_RT[0] + TASK_RT[1] + TASK_RT[2];
    const CPU_UTIL: u32 = (TASK_RT_TOT * 100) / HYPERPERIOD;
    const US_TO_CC: u32 = 100;

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let i2c = I2c::init(4);
        let cfg = CfgRegs::init();
        let (_ibx, mut obx) = unsafe { Mailbox::instance() }.split();

        // Read platform configuration
        let commit = cfg.commit();
        let intc_edfic = cfg.intc_edfic();
        let intc_name = if intc_edfic { "EDFIC" } else { "CLIC " };

        if !cfg.intc_valid() {
            panic!("INTC mismatch! Recompile HW or check feature flags");
        }

        // Enable dynamic interrupt behavior when using EDFIC
        if intc_edfic {
            cfg.enable_dynamic_intc();
        }

        sprintln!("[rtprof] interrupt controller benchmark");
        sprintln!(
            "Platform - HW commit   : {:x}, intc: {},        CPU Frequency (MHz): {}",
            commit,
            intc_name,
            CPU_FREQ_HZ / 1_000_000,
        );
        sprintln!(
            "Testcase - runtime (ms): {:7}, load: (0..100): {},    Hyperperiod (us): {}",
            RT,
            LF,
            HYPERPERIOD,
        );

        for i in 0..TASK_SET_SIZE {
            sprintln!(
                "Task {i}: F (per HP): {}, Total runtime (us): {}",
                TASK_FREQ[i],
                TASK_RT[i]
            )
        }

        sprintln!(
            "Theoretical CPU utilization: {} us/{} us = {} % \n",
            TASK_RT_TOT,
            HYPERPERIOD,
            CPU_UTIL
        );

        let task_dl_base = 0x2_0000;

        for i in 0..TASK_SET_SIZE {
            obx.send(task_dl_base + i as u32, TASK_SET[i].deadline_us * US_TO_CC);
        }

        // 1 tick == 1 us
        MTimer::with_clkdiv(100).start(RT.millis());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            //Timer::init::<TIMER1_ADDR>().into_periodic(),
            //Timer::init::<TIMER2_ADDR>().into_periodic(),
        ];

        for i in 0..1 {
            //TASK_SET_SIZE {
            timers[i].set_period(TASK_SET[i].period_us.micros());
        }

        timers.iter_mut().for_each(Periodic::start);

        clear_perf_counters();
        rtprof_start_full();

        Shared { i2c }
    }

    #[task(binds = MachineTimer, priority = 0xff)]
    struct Finish {}
    impl RticTask for Finish {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_stop();

            let now = MTimer::instance().now().as_ticks();
            let minstret = minstret::read64();
            let mcycle = mcycle::read64();

            sprintln!(
                "True CPU utilization: {} %, instructions: {}",
                ((mcycle * 100) / now),
                minstret
            );
            #[cfg(feature = "obs")]
            obs_trace::obs_dump!(obs_trace::TsUnit::Micros);
            signal_pass(None);
        }
    }

    // DL: 30 Prio: 255 - 30 = 225
    #[task(binds = Timer0Cmp, priority = 225, shared = [i2c])]
    struct Timer0 {}
    impl RticTask for Timer0 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(0);

            let mut rbuf:[u8; 4] = [0, 0,0,0];
            self.shared().i2c.lock(|i2c| {
                i2c.read(0b1 as u8, &mut rbuf);
            });
            sprintln!("{}", u32::from_le_bytes(rbuf));

            rtprof_end_task(0);
        }
    }

    // DL: 30 Prio: 255 - 50 = 205
    #[task(binds = Timer1Cmp, priority = 205)]
    struct Timer1 {}
    impl RticTask for Timer1 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(1);

            //run_us(TASK_SET[1].runtime_us);

            rtprof_end_task(1);
        }
    }

    // DL: 30 Prio: 255 - 150 = 225
    #[task(binds = Timer2Cmp, priority = 105)]
    struct Timer2 {}
    impl RticTask for Timer2 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(2);

            //run_us(TASK_SET[2].runtime_us);

            rtprof_end_task(2);
        }
    }
}
