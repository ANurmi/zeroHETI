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
    fn capture_irq_ts() {
        // Special logic to directly capture mtime into edf_ts 64-bit register
        unsafe { core::arch::asm!("csrrwi x0, 0x367, 1") };
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

    const TIMER0_PER_US: u32 = 2 * (100 - LF);
    const TIMER1_PER_US: u32 = 5 * (100 - LF);
    const TIMER2_PER_US: u32 = 9 * (100 - LF);

    const TIMER0_LOAD: u32 = 200; // ~12 us
    const TIMER1_LOAD: u32 = 500; // ~30 us
    const TIMER2_LOAD: u32 = 900; // ~54 us

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

        MTimer::with_clkdiv(500)
            .into_oneshot()
            .start(RT.millis() / 4); // HACK

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

            let now = MTimer::instance().into_oneshot().duration().as_ticks();
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

    #[task(binds = Timer0Cmp, priority = 150)]
    struct Timer0 {}
    impl RticTask for Timer0 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            // TODO: read and clear csr_edf_count
            // csrrw rd, csr_edf_count, x0
            let count: usize;
            let ts_lo: usize;
            let ts_hi: usize;

            unsafe { core::arch::asm!("csrrw {0}, 0x366, x0", out(reg) count) };
            unsafe { core::arch::asm!("csrrs {0}, 0x362, x0", out(reg) ts_lo) };
            unsafe { core::arch::asm!("csrrs {0}, 0x363, x0", out(reg) ts_hi) };

            capture_irq_ts();

            rtprof_start_task(0);
            // FUNCTIONAL ISR

            asm_delay(TIMER0_LOAD);

            // ISR END
            rtprof_end_task(0);

            unsafe { core::arch::asm!("csrw 0x362, {0}", in(reg) ts_lo) };
            unsafe { core::arch::asm!("csrw 0x363, {0}", in(reg) ts_hi) };
            unsafe { core::arch::asm!("csrw 0x366, {0}", in(reg) count) };
        }
    }

    #[task(binds = Timer1Cmp, priority = 100)]
    struct Timer1 {}
    impl RticTask for Timer1 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let count: usize;
            let ts_lo: usize;
            let ts_hi: usize;

            unsafe { core::arch::asm!("csrrw {0}, 0x366, x0", out(reg) count) };
            unsafe { core::arch::asm!("csrrs {0}, 0x362, x0", out(reg) ts_lo) };
            unsafe { core::arch::asm!("csrrs {0}, 0x363, x0", out(reg) ts_hi) };

            capture_irq_ts();
            rtprof_start_task(1);

            asm_delay(TIMER1_LOAD);

            rtprof_end_task(1);

            unsafe { core::arch::asm!("csrw 0x362, {0}", in(reg) ts_lo) };
            unsafe { core::arch::asm!("csrw 0x363, {0}", in(reg) ts_hi) };
            unsafe { core::arch::asm!("csrw 0x366, {0}", in(reg) count) };
        }
    }

    #[task(binds = Timer2Cmp, priority = 50)]
    struct Timer2 {}
    impl RticTask for Timer2 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let count: usize;
            let ts_lo: usize;
            let ts_hi: usize;

            unsafe { core::arch::asm!("csrrw {0}, 0x366, x0", out(reg) count) };
            unsafe { core::arch::asm!("csrrs {0}, 0x362, x0", out(reg) ts_lo) };
            unsafe { core::arch::asm!("csrrs {0}, 0x363, x0", out(reg) ts_hi) };

            capture_irq_ts();
            rtprof_start_task(2);

            asm_delay(TIMER2_LOAD);

            rtprof_end_task(2);

            unsafe { core::arch::asm!("csrw 0x362, {0}", in(reg) ts_lo) };
            unsafe { core::arch::asm!("csrw 0x363, {0}", in(reg) ts_hi) };
            unsafe { core::arch::asm!("csrw 0x366, {0}", in(reg) count) };
        }
    }
}
