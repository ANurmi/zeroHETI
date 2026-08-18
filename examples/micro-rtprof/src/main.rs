#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;
#[rtic::app(device = bsp /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/)]

mod app {
    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        asm_delay,
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
    };

    use fugit::{ExtU32, ExtU64};

    const CFG_BASE_ADDR: usize = 0x0000_4000;
    const CFG_TASK_OFFS: usize = 0x0000_0100;

    fn clear_perf_counters() {
        unsafe {
            minstret::write(0);
            mcycle::write(0);
            minstreth::write(0);
            mcycleh::write(0);
        }
    }

    const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
    const RT: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

    const TIMER0_PER_US: u32 = 2 * (100 - LF);
    const TIMER1_PER_US: u32 = 5 * (100 - LF);
    const TIMER2_PER_US: u32 = 9 * (100 - LF);

    const TIMER0_LOAD: u32 = 200;
    const TIMER1_LOAD: u32 = 500;
    const TIMER2_LOAD: u32 = 900;

    const US_TO_CC: u32 = 100;

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
        let rd = mmio::read_u32(CFG_BASE_ADDR);
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        let is_intc_edfic = (cfg & 0b1) == 0b1;
        let intc_name = if is_intc_edfic { "EDFIC" } else { "CLIC" };

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
            "- HW commit: {:x}, intc: {}, CPU (MHz): {}, runtime (ms): {}, load: (0..100): {}",
            rd,
            intc_name,
            CPU_FREQ_HZ / 1_000_000,
            RT,
            LF
        );

        let task_dl_base = 0x1_0000;

        // Set period == deadline for periodic work
        obx.send(task_dl_base + 0, TIMER0_PER_US * US_TO_CC);
        obx.send(task_dl_base + 1, TIMER1_PER_US * US_TO_CC);
        obx.send(task_dl_base + 2, TIMER2_PER_US * US_TO_CC);

        // Setup mtimer to trigger `Finish` task
        MTimer::with_clkdiv(250).into_oneshot().start(RT.millis());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
        ];

        timers[0].set_period(TIMER0_PER_US.micros());
        timers[1].set_period(TIMER1_PER_US.micros());
        timers[2].set_period(TIMER2_PER_US.micros());

        timers.iter_mut().for_each(Periodic::start);

        clear_perf_counters();
        // Scoreboard enable
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

            let now = MTimer::instance().into_oneshot().duration().ticks();
            let minstret = minstret::read64();
            let mcycle = mcycle::read64();

            sprintln!(
                "CPU util: {}%, instructions: {}",
                ((mcycle * 100) / now),
                minstret
            );
            signal_pass(None);
        }
    }

    #[task(binds = Timer0Cmp, priority = 0xAA)]
    struct Timer0 {}
    impl RticTask for Timer0 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 4, 1);
            asm_delay(TIMER0_LOAD);
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 4, 0);
        }
    }

    #[task(binds = Timer1Cmp, priority = 0x0A)]
    struct Timer1 {}
    impl RticTask for Timer1 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 8, 1);
            asm_delay(TIMER1_LOAD);
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 8, 0);
        }
    }

    #[task(binds = Timer2Cmp, priority = 0x08)]
    struct Timer2 {}
    impl RticTask for Timer2 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 12, 1);
            asm_delay(TIMER2_LOAD);
            mmio::write_u32(CFG_BASE_ADDR + CFG_TASK_OFFS + 12, 0);
        }
    }
}
