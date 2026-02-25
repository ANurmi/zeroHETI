#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fclic-hetic, -Fclic-clic, -Fclic-edfic"
);

use bsp::rt as _;
#[rtic::app(device = bsp)]
mod app {
    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::*,
        i2c::{self, I2c},
        mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mtimer::{self, *},
        riscv, sprintln,
        tb::signal_pass,
        timer_group::{Periodic, Timer},
    };
    use core::i16;
    use fugit::{ExtU32, ExtU64};
    use motor_control::mailbox::{Mailbox, Motor::*};

    struct SimParams {
        hyperperiod_ms: u64,
    }

    const SIM_PARAMS: SimParams = SimParams { hyperperiod_ms: 20 };

    const REP_TASK_PER_US: u32 = 4000;
    const REP_TASK_OFS_US: u32 = 3900;

    struct Motor {
        stat: u8,
        ctrl: u8,
        tune: u8,
    }

    const M0_ADDR: Motor = Motor {
        stat: 1,
        ctrl: 2,
        tune: 3,
    };
    const M1_ADDR: Motor = Motor {
        stat: 4,
        ctrl: 5,
        tune: 6,
    };
    const M2_ADDR: Motor = Motor {
        stat: 7,
        ctrl: 8,
        tune: 9,
    };
    const M3_ADDR: Motor = Motor {
        stat: 10,
        ctrl: 11,
        tune: 12,
    };

    #[cfg(feature = "intc-edfic")]
    const MBX_DL_US: u32 = 5000;
    #[cfg(feature = "intc-edfic")]
    const WRN_DL_US: u32 = 4000;
    #[cfg(feature = "intc-edfic")]
    const REP_DL_US: u32 = 3000;

    // Global variables
    static mut MBX: Mailbox = unsafe { Mailbox::instance() };

    static mut SPEED_REAL: [u32; 4] = [0, 0, 0, 0];
    static mut VOLTAGE_TARGET: [u32; 4] = [0, 0, 0, 0];

    static mut INTEGRAL: [i32; 4] = [0, 0, 0, 0];
    static mut PREV_ERR: [i32; 4] = [0, 0, 0, 0];

    static mut START_TIME: Option<u64> = None;
    static mut END_TIME: Option<u64> = None;

    #[shared]
    struct Shared {
        end_timer: mtimer::OneShot,
        i2c: i2c::I2c,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let riscv_isa = core::env!("RISCV_ISA");
        sprintln!("[Motor control demo] ISA = {riscv_isa}");

        let mut i2c = I2c::init(4);

        // HACK: clear mintstatus
        unsafe { bsp::register::mintstatus::write(0.into()) };

        let mut end_timer = MTimer::instance().into_oneshot();

        unsafe { START_TIME.replace(end_timer.counter()) };

        end_timer.start(SIM_PARAMS.hyperperiod_ms.millis());

        let timers = &mut [
            Timer::init::<TIMER0_ADDR>().into_periodic(),
            Timer::init::<TIMER1_ADDR>().into_periodic(),
            Timer::init::<TIMER2_ADDR>().into_periodic(),
            Timer::init::<TIMER3_ADDR>().into_periodic(),
        ];

        timers[0].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());
        timers[1].set_period_offset(
            REP_TASK_PER_US.micros(),
            REP_TASK_OFS_US.micros() - (1 * 1000u32).micros(),
        );
        timers[2].set_period_offset(
            REP_TASK_PER_US.micros(),
            REP_TASK_OFS_US.micros() - (2 * 1000u32).micros(),
        );
        timers[3].set_period_offset(
            REP_TASK_PER_US.micros(),
            REP_TASK_OFS_US.micros() - (3 * 1000u32).micros(),
        );

        unsafe {
            // Clear instruction & cycle counters
            riscv::register::minstret::write(0);
            riscv::register::mcycle::write(0);
            riscv::register::minstreth::write(0);
            riscv::register::mcycleh::write(0);
        }

        // Start periodic timers
        timers.iter_mut().for_each(Periodic::start);

        // Start sim
        unsafe {
            i2c.write(0x0, &[1]);
            riscv::interrupt::enable();
        }

        Shared { end_timer, i2c }
    }

    #[task(binds = Timer0Cmp, priority = 0x88, shared = [i2c])]
    struct ReadM0 {}

    impl RticTask for ReadM0 {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(M0_ADDR.stat, &mut rbuf));

            let m0_speed = u32::from_le_bytes(rbuf);
            let time = MTimer::instance().counter();

            // SAFETY: there are no other users of SPEED_REAL[0]
            unsafe { SPEED_REAL[0] = m0_speed };
            riscv::interrupt::free(||
            // SAFETY: other users of MBX are excluded
            unsafe { MBX.write_time_and_stat(time, m0_speed as u32, M0) });
        }
    }

    #[task(binds = Timer1Cmp, priority = 0x88)]
    struct ReadM1 {}
    impl RticTask for ReadM1 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(M1_ADDR.stat, &mut rbuf));

            let m1_speed = u32::from_le_bytes(rbuf);
            let time = MTimer::instance().counter();

            // SAFETY: there are no other users of SPEED_REAL[1]
            unsafe { SPEED_REAL[1] = m1_speed };
            riscv::interrupt::free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m1_speed as u32, M1) });
        }
    }

    #[task(binds = Timer2Cmp, priority = 0x88)]
    struct ReadM2 {}
    impl RticTask for ReadM2 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
         unsafe { I2c::instance() }.read(M2_ADDR.stat, &mut rbuf));

            let m2_speed = u32::from_le_bytes(rbuf);
            let time = MTimer::instance().counter();

            // SAFETY: there are no other users of SPEED_REAL[2]
            unsafe { SPEED_REAL[2] = m2_speed };
            riscv::interrupt::free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m2_speed as u32, M2)});
        }
    }

    #[task(binds = Timer3Cmp, priority = 0x88)]
    struct ReadM3 {}
    impl RticTask for ReadM3 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
         unsafe { I2c::instance() }.read(M3_ADDR.stat, &mut rbuf));

            let m3_speed = u32::from_le_bytes(rbuf);
            let time = MTimer::instance().counter();

            // SAFETY: there are no other users of SPEED_REAL[3]
            unsafe { SPEED_REAL[3] = m3_speed };
            riscv::interrupt::free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m3_speed as u32, M3)});
        }
    }

    #[task(binds = Mbx, priority = 3)]
    struct GetMail;
    impl RticTask for GetMail {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            // SAFETY: the inbox is not read by any other context
            let mail = unsafe { MBX.read_inbox() };
            let bytes: [u8; 4] = mail.to_be_bytes();

            for i in 0usize..4 {
                riscv::interrupt::free(|| {
                    // SAFETY: other users of I2C are excluded
                    unsafe { I2c::instance() }.write((2 + i * 3) as u8, &[0u8, bytes[i]]);
                    let target_speed = (bytes[i] as u32) << 8;
                    // SAFETY: it is not significant whether motors read the newest or the last
                    // value from VOLTAGE_TARGET[i], or if they are mixed up.
                    unsafe { VOLTAGE_TARGET[i] = target_speed };
                });
            }

            unsafe {
                MBX.ack_irq();
            }
        }
    }

    #[task(binds = Ext0, priority = 0x10)]
    struct TuneM0;
    impl RticTask for TuneM0 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(M0_ADDR.stat, &mut rbuf));

            let m0_speed_now = u32::from_le_bytes(rbuf);
            let bytes: [u8; 2] = compute_control(0, m0_speed_now).to_le_bytes();
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(M0_ADDR.tune, &bytes));
        }
    }

    #[task(binds = Ext1, priority = 0x10)]
    struct TuneM1;
    impl RticTask for TuneM1 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(M1_ADDR.stat, &mut rbuf));

            let m1_speed_now = u32::from_le_bytes(rbuf);
            let bytes: [u8; 2] = compute_control(1, m1_speed_now).to_le_bytes();
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(M1_ADDR.tune, &bytes));
        }
    }

    #[task(binds = Ext2, priority = 0x10)]
    struct TuneM2;
    impl RticTask for TuneM2 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(M2_ADDR.stat, &mut rbuf));

            let m2_speed_now = u32::from_le_bytes(rbuf);
            let bytes: [u8; 2] = compute_control(2, m2_speed_now).to_le_bytes();
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(M2_ADDR.tune, &bytes));
        }
    }

    #[task(binds = Ext3, priority = 0x10)]
    struct TuneM3;
    impl RticTask for TuneM3 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(M3_ADDR.stat, &mut rbuf));

            let m3_speed_now = u32::from_le_bytes(rbuf);
            let bytes: [u8; 2] = compute_control(3, m3_speed_now).to_le_bytes();
            riscv::interrupt::free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(M3_ADDR.tune, &bytes));
        }
    }

    #[inline]
    /// Compute tuning voltage to control motor power.
    fn compute_control(idx: usize, speed_now: u32) -> i16 {
        // Resistance in mOhm
        let res = 10_000;
        let v_target = riscv::interrupt::free(||
        // SAFETY: other users of VOLTAGE_TARGET[idx] are excluded
        unsafe { VOLTAGE_TARGET }[idx]);
        let p_target = u32::pow(v_target, 2) / res; // mW

        // Assume Power (mW) and Speed (RPM) directly correlated
        let error = p_target as i32 - speed_now as i32;

        let mut v_out: i16 = usqrt4((error / res as i32).abs() as u32) as i16;

        if error < 0 {
            v_out = v_out * (-1);
        };

        v_out
        /*
           let KI_DEN: i32 = 1;*
           //let res: i32 = ((KP_NOM * err) / KP_DEN) + ((KI_DEN * INTEGRAL[idx]) /
           // KI_DEN);
           // KP = 2, KI = 1, KD = 0.2

           let KP = 2;
           let KI = 2;
           //  KD == 0.2
           let INV_KD = 5;


           // Additional correction term
           INTEGRAL[idx] += error;
           let integral = INTEGRAL[idx];
           let derivative = error - PREV_ERR[idx];

           PREV_ERR[idx] = error;

           let p_corr = KP * error + KI * integral + (derivative / INV_KD);

           let mut v_tune: i16 = usqrt4((p_corr/(res)) as u32) as i16;

           if error < 0 {
               v_tune = v_tune * (-1);
           };

           /*
           let mut neg = false;


           //v_tune *= KV;
           if (neg & ((v_tune as i16) > 0)) {
               v_tune = i16::MIN as i32;
           }

           if (!neg & ((v_tune as i16) < 0)) {
               v_tune = i16::MAX as i32;
           }
        */

           v_tune */
    }

    #[inline]
    fn usqrt4(val: u32) -> u32 {
        // Starting point is relatively unimportant
        let mut a: u32 = 1255;
        let mut b: u32;

        // Avoid division by zero
        if val < 2 {
            return val;
        };

        // 4 iterations
        for _ in 0..4 {
            b = val / a;
            a = (a + b) / 2;
        }

        a
    }

    /*
    #[cfg_attr(not(feature = "intc-edfic"), task(binds = MachineTimer = 0xff, priority=1, shared=[mtimer]))]
    #[cfg_attr(feature = "intc-edfic", task(binds = MachineTimer = 0x0, priority=1, shared=[mtimer]))]
    */
    #[task(binds = MachineTimer, priority = 1)]
    struct Finish;
    impl RticTask for Finish {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            riscv::interrupt::disable();

            // SAFETY: interrupts are now disabled

            unsafe { END_TIME.replace(MTimer::instance().counter().into()) };

            // Explicitly terminate simulation to print task log
            unsafe { I2c::instance() }.write(0x0, &[0]);

            let instret = riscv::register::minstret::read64();
            let active_time_cc = riscv::register::mcycle::read64();

            sprintln!("Instructions retired: {instret}, cycles: {active_time_cc}");

            let total_time_cc = unsafe { END_TIME.unwrap() - START_TIME.unwrap() };

            // For microseconds, use the following:
            /*
            let active_time_us = active_time_cc / (1_000 * 1_000 * CPU_FREQ_HZ as u64);
            let total_time_us = total_time_cc / (1_000 * 1_000 * CPU_FREQ_HZ as u64);
            sprintln!(
                "Total time (us): {}, active time (us): {},",
                total_time_us,
                active_time_us
            );
            */
            sprintln!(
                "Total time (cc): {}, active time (cc): {},",
                total_time_cc,
                active_time_cc
            );
            sprintln!(
                "CPU utilization: {}%",
                (active_time_cc * 100) / total_time_cc
            );

            signal_pass(None);
        }
    }
}
