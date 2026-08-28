#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;

#[cfg(feature = "obs")]
#[cfg_attr(feature = "obs", rtic::app(device = bsp, obs = obs_trace::Obs /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
#[cfg_attr(not(feature = "obs"), rtic::app(device = bsp /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
mod app {
    use bsp::{
        apb_uart::ApbUart,
        asm_delay,
        fugit::{ExtU32, ExtU64},
        i2c::{self, I2c},
        mailbox::Mailbox,
        mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR},
        mmio,
        mtimer::*,
        parse_u32,
        register::{mcycle, mcycleh, minstret, minstreth},
        sprintln,
        tb::signal_pass,
        timer_group::{Periodic, Timer},
        CPU_FREQ_HZ,
    };

    const CFG_BASE_ADDR: usize = 0x0000_4000;
    const CFG_TASK_OFFS: usize = 0x0000_0100;

    #[inline]
    fn clear_perf_counters() {
        unsafe {
            minstret::write(0);
            mcycle::write(0);
            minstreth::write(0);
            mcycleh::write(0);
        }
    }

    #[inline]
    fn rtprof_start_task(idx: usize) {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + (4 + 4 * idx), 1);
    }

    #[inline]
    fn rtprof_end_task(idx: usize) {
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + (4 + 4 * idx), 0);
    }

    const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
    const RT: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

    struct Task {
        period_us: u32,
        deadline_us: u32,
        runtime_us: u32,
    }

    const TASK_SET: [Task; 3] = [
        Task {
            period_us: 30,
            deadline_us: 50,
            runtime_us: 8 * LF / 100,
        },
        Task {
            period_us: 66,
            deadline_us: 100,
            runtime_us: 30 * LF / 100,
        },
        Task {
            period_us: 170,
            deadline_us: 150,
            runtime_us: 50 * LF / 100,
        },
    ];

    const HYPERPERIOD: u32 = lcm3(
        TASK_SET[0].period_us,
        TASK_SET[1].period_us,
        TASK_SET[2].period_us,
    );

    const US_TO_CC: u32 = 100;

    const fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }

    const fn lcm2(a: u32, b: u32) -> u32 {
        if a == 0 || b == 0 {
            0
        } else {
            (a / gcd(a, b)) * b
        }
    }

    const fn lcm3(a: u32, b: u32, c: u32) -> u32 {
        lcm2(lcm2(a, b), c)
    }

    #[inline]
    fn run_us(rt: u32) {
        // Experimentally measured coefficient
        let k = 16;
        asm_delay(rt * k);
    }

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let i2c = I2c::init(4);
        let (_ibx, mut obx) = unsafe { Mailbox::instance() }.split();

        // Read platform configuration
        let commit = mmio::read_u32(CFG_BASE_ADDR);
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        let is_intc_edfic = (cfg & 0b1) == 0b1;
        let intc_name = if is_intc_edfic { "EDFIC" } else { "CLIC " };

        #[cfg(any(feature = "intc-edfic", feature = "intc-clic"))]
        {
            #[cfg(feature = "intc-clic")]
            if is_intc_edfic {
                panic!("INTC=EDFIC, expected CLIC");
            }
            #[cfg(feature = "intc-edfic")]
            {
                // EDFIC-specific: enable mtime to interrupt controller
                mmio::write_u32(CFG_BASE_ADDR + 8, 0x1);

                if !is_intc_edfic {
                    panic!("INTC=!EDFIC, expected EDFIC");
                }
            }
        }

        sprintln!("[micro-rtprof] interrupt controller microbenchmark");

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

        // How often a task is run within hyperperiod
        let f_t0 = HYPERPERIOD / TASK_SET[0].period_us;
        let f_t1 = HYPERPERIOD / TASK_SET[1].period_us;
        let f_t2 = HYPERPERIOD / TASK_SET[2].period_us;

        let rt_t0 = f_t0 * TASK_SET[0].runtime_us;
        let rt_t1 = f_t1 * TASK_SET[1].runtime_us;
        let rt_t2 = f_t2 * TASK_SET[2].runtime_us;

        sprintln!(
            "Task 0: F (per HP): {}, Total runtime (us): {}",
            f_t0,
            rt_t0
        );
        sprintln!(
            "Task 1: F (per HP): {}, Total runtime (us): {}",
            f_t1,
            rt_t1
        );
        sprintln!(
            "Task 2: F (per HP): {}, Total runtime (us): {}",
            f_t2,
            rt_t2
        );

        let rt_tot = rt_t0 + rt_t1 + rt_t2;
        let util = (rt_tot * 100) / HYPERPERIOD;

        sprintln!(
            "Theoretical CPU utilization: {} us/{} us = {} % \n",
            rt_tot,
            HYPERPERIOD,
            util
        );

        let task_dl_base = 0x1_0000;

        obx.send(task_dl_base + 0, TASK_SET[0].deadline_us * US_TO_CC);
        obx.send(task_dl_base + 1, TASK_SET[1].deadline_us * US_TO_CC);
        obx.send(task_dl_base + 2, TASK_SET[2].deadline_us * US_TO_CC);

        // 1 tick == 1 us
        MTimer::with_clkdiv(100).into_oneshot().start(RT.millis());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
        ];

        timers[0].set_period(TASK_SET[0].period_us.micros());
        timers[1].set_period(TASK_SET[1].period_us.micros());
        timers[2].set_period(TASK_SET[2].period_us.micros());

        timers.iter_mut().for_each(Periodic::start);

        clear_perf_counters();
        // Scoreboard enable
        // TODO: Generalize for full rt-prof
        mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS, 1);

        Shared { i2c }
    }

    #[task(binds = MachineTimer, priority = 0xff)]
    struct Finish {}
    impl RticTask for Finish {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            // Scoreboard disable
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS, 0);

            let now = MTimer::instance().into_oneshot().duration().as_ticks();
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

    #[task(binds = Timer0Cmp, priority = 133)]
    struct Timer0 {}
    impl RticTask for Timer0 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(0);
            // FUNCTIONAL ISR

            run_us(TASK_SET[0].runtime_us);

            // ISR END
            rtprof_end_task(0);
        }
    }

    #[task(binds = Timer1Cmp, priority = 100)]
    struct Timer1 {}
    impl RticTask for Timer1 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(1);

            run_us(TASK_SET[1].runtime_us);

            rtprof_end_task(1);
        }
    }

    #[task(binds = Timer2Cmp, priority = 67)]
    struct Timer2 {}
    impl RticTask for Timer2 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(2);

            run_us(TASK_SET[2].runtime_us);

            rtprof_end_task(2);
        }
    }
}
