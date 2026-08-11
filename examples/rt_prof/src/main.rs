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
        mtimer::{self, *},
        sprintln,
        tb::signal_pass,
    };

    use fugit::{ExtU32, ExtU64};

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let i2c = I2c::init(4);

        sprintln!("Start");
        MTimer::instance().into_oneshot().start(1000u64.micros());
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
            asm_delay(100);
            signal_pass(None);
        }
    }
}
