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
        mmap::apb_timer::TIMER0_ADDR,
        mmio,
        mailbox::Mailbox,
        mtimer::{self, *},
        riscv::{self},
        sprintln,
        tb::signal_pass,
        timer_group::{Periodic, Timer},
    };

    use fugit::{ExtU32, ExtU64};

    const CFG_BASE_ADDR: usize = 0x0000_3500;
    
    fn clear_perf_counters() {
        unsafe {
            riscv::register::minstret::write(0);
            riscv::register::mcycle::write(0);
            riscv::register::minstreth::write(0);
            riscv::register::mcycleh::write(0);
        }
    }

    const TIMER_PER_US: u32 = 100;

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let i2c = I2c::init(4);
        let (_ibx, mut obx) = unsafe { Mailbox::instance() }.split();

        sprintln!("Start, CPU (MHz): {}", CPU_FREQ_HZ / 1_000_000);

        let rd = mmio::read_u32(CFG_BASE_ADDR);

        obx.send(67,67);
        
        // scb enable
        mmio::write_u32(CFG_BASE_ADDR + 4, 1);
        
        /*
        mmio::write_u32(CFG_BASE_ADDR + 8, 1);
        mmio::write_u32(CFG_BASE_ADDR + 12, 1);
        mmio::write_u32(CFG_BASE_ADDR + 16, 1);
        mmio::write_u32(CFG_BASE_ADDR + 20, 1);
        */
        sprintln!("HW build commit: {:X}", rd);

        MTimer::instance().into_oneshot().start(1000u64.micros());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            /*
                Timer::init::<TIMER1_ADDR>().into_periodic(),
                Timer::init::<TIMER2_ADDR>().into_periodic(),
                Timer::init::<TIMER3_ADDR>().into_periodic(),
            */
        ];

        timers
            .iter_mut()
            .for_each(|t| t.set_period(TIMER_PER_US.micros()));

        timers.iter_mut().for_each(Periodic::start);

        clear_perf_counters();

        Shared { i2c }
    }

    #[task(binds = MachineTimer, priority = 0xff)]
    struct StartSim {
        mtimer: mtimer::OneShot,
    }
    impl RticTask for StartSim {
        fn init() -> Self {
            let mtimer = MTimer::instance().into_oneshot();
            Self { mtimer }
        }
        fn exec(&mut self) {
            sprintln!("End");

            // Delay to let UART complete
            asm_delay(10_000);
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
            mmio::write_u32(CFG_BASE_ADDR + 8, 1);
            sprintln!("T0");
            mmio::write_u32(CFG_BASE_ADDR + 8, 0);
        }
    }
}
