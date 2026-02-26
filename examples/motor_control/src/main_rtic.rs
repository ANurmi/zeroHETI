#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fclic-hetic, -Fclic-clic, -Fclic-edfic"
);

use bsp::rt as _;
#[rtic::app(device = bsp/*, dispatchers = [MachineSoft]*/)]
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
    use motor_control::{
        I2C_ADDRS,
        mailbox::{Mailbox, Motor::*},
    };

    struct SimParams {
        hyperperiod_ms: u64,
        rep_task_per_us: u32,
        rep_task_ofs_us: u32,
    }

    const SIM_PARAMS: SimParams = SimParams {
        hyperperiod_ms: 20,
        rep_task_per_us: 4000,
        rep_task_ofs_us: 3900,
    };

    #[cfg(feature = "intc-edfic")]
    const MBX_DL_US: u32 = 5000;
    #[cfg(feature = "intc-edfic")]
    const WRN_DL_US: u32 = 4000;
    #[cfg(feature = "intc-edfic")]
    const REP_DL_US: u32 = 3000;

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
        mbx: Mailbox,
        /// Voltage target
        v_target: [u32; 4],
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let riscv_isa = core::env!("RISCV_ISA");
        sprintln!("[Motor control demo] ISA = {riscv_isa}");

        let i2c = I2c::init(4);
        let mbx = unsafe { Mailbox::instance() };
        let v_target = [0; 4];

        // HACK: use mtimer to start the sim, CLIC cannot enable IRQ after pend,
        // see: https://github.com/ANurmi/zeroHETI/issues/42
        // It takes about 40 us for the rest of setup to complete. Let's leave a
        // little margin.
        MTimer::instance().into_oneshot().start(100u64.micros());
        //StartSim::spawn(()).unwrap();

        Shared { i2c, mbx, v_target }
    }

    //#[sw_task(priority = 0xff, shared=[i2c])]
    #[task(binds = MachineTimer, priority = 0xff, shared=[i2c])]
    struct StartSim {
        mtimer: mtimer::OneShot,
        start_time: Option<u64>,
    }
    //impl RticSwTask for StartSim {
    impl RticTask for StartSim {
        //type SpawnInput = ();

        fn init() -> Self {
            let mtimer = MTimer::instance().into_oneshot();
            Self {
                mtimer,
                start_time: None,
            }
        }

        fn exec(&mut self /* , _: () */) {
            match self.start_time.as_mut() {
                None => {
                    let timers = &mut [
                        Timer::init::<TIMER0_ADDR>().into_periodic(),
                        Timer::init::<TIMER1_ADDR>().into_periodic(),
                        Timer::init::<TIMER2_ADDR>().into_periodic(),
                        Timer::init::<TIMER3_ADDR>().into_periodic(),
                    ];

                    timers[0].set_period_offset(
                        SIM_PARAMS.rep_task_per_us.micros(),
                        SIM_PARAMS.rep_task_ofs_us.micros(),
                    );
                    timers[1].set_period_offset(
                        SIM_PARAMS.rep_task_per_us.micros(),
                        SIM_PARAMS.rep_task_ofs_us.micros() - (1 * 1000u32).micros(),
                    );
                    timers[2].set_period_offset(
                        SIM_PARAMS.rep_task_per_us.micros(),
                        SIM_PARAMS.rep_task_ofs_us.micros() - (2 * 1000u32).micros(),
                    );
                    timers[3].set_period_offset(
                        SIM_PARAMS.rep_task_per_us.micros(),
                        SIM_PARAMS.rep_task_ofs_us.micros() - (3 * 1000u32).micros(),
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

                    // Start hyperperiod timer
                    self.start_time.replace(self.mtimer.counter());
                    self.mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

                    // Start sim
                    self.shared().i2c.lock(|i2c| i2c.write(0x0, &[1]));
                }
                Some(start_time) => {
                    riscv::interrupt::disable();

                    // SAFETY: interrupts are now disabled

                    let end_time: u64 = MTimer::instance().counter().into();

                    // Explicitly terminate simulation to print task log
                    unsafe { I2c::instance() }.write(0x0, &[0]);

                    let instret = riscv::register::minstret::read64();
                    let active_time_cc = riscv::register::mcycle::read64();

                    sprintln!("Instructions retired: {instret}, cycles: {active_time_cc}");

                    let total_time_cc = end_time - *start_time;

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
    }

    #[task(binds = Timer0Cmp, priority = 0x88, shared = [i2c, mbx])]
    struct ReadM0 {
        speed_real: u32,
    }

    impl RticTask for ReadM0 {
        fn init() -> Self {
            Self { speed_real: 0 }
        }

        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[0].stat, &mut rbuf));

            let m0_speed = u32::from_le_bytes(rbuf);
            self.speed_real = m0_speed;

            let time = MTimer::instance().counter();
            self.shared()
                .mbx
                .lock(|mbx| mbx.write_time_and_stat(time, m0_speed as u32, M0));
        }
    }

    #[task(binds = Timer1Cmp, priority = 0x88, shared = [i2c, mbx])]
    struct ReadM1 {
        speed_real: u32,
    }
    impl RticTask for ReadM1 {
        fn init() -> Self {
            Self { speed_real: 0 }
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];

            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[1].stat, &mut rbuf));

            let m1_speed = u32::from_le_bytes(rbuf);
            self.speed_real = m1_speed;

            let time = MTimer::instance().counter();
            self.shared()
                .mbx
                .lock(|mbx| mbx.write_time_and_stat(time, m1_speed as u32, M1));
        }
    }

    #[task(binds = Timer2Cmp, priority = 0x88, shared = [i2c, mbx])]
    struct ReadM2 {
        speed_real: u32,
    }
    impl RticTask for ReadM2 {
        fn init() -> Self {
            Self { speed_real: 0 }
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[2].stat, &mut rbuf));

            let m2_speed = u32::from_le_bytes(rbuf);
            self.speed_real = m2_speed;

            let time = MTimer::instance().counter();
            self.shared()
                .mbx
                .lock(|mbx| mbx.write_time_and_stat(time, m2_speed as u32, M2));
        }
    }

    #[task(binds = Timer3Cmp, priority = 0x88, shared = [i2c, mbx])]
    struct ReadM3 {
        speed_real: u32,
    }
    impl RticTask for ReadM3 {
        fn init() -> Self {
            Self { speed_real: 0 }
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[3].stat, &mut rbuf));

            let m3_speed = u32::from_le_bytes(rbuf);
            self.speed_real = m3_speed;

            let time = MTimer::instance().counter();
            self.shared()
                .mbx
                .lock(|mbx| mbx.write_time_and_stat(time, m3_speed as u32, M3));
        }
    }

    #[task(binds = Mbx, priority = 3, shared = [i2c, v_target])]
    struct GetMail;
    impl RticTask for GetMail {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            // SAFETY: the inbox is not read by any other context
            let mail = unsafe { Mailbox::instance() }.read_inbox();
            let bytes: [u8; 4] = mail.to_be_bytes();

            for i in 0usize..4 {
                self.shared().i2c.lock(|i2c| {
                    i2c.write(I2C_ADDRS.motors[i].ctrl, &[0u8, bytes[i]]);
                    let target_speed = (bytes[i] as u32) << 8;
                    self.shared()
                        .v_target
                        .lock(|v_target| v_target[i] = target_speed);
                });
            }

            // SAFETY: the mailbox ACK_IRQ is not interacted with by any other
            // part of the code, and it does not interfere with other Mailbox
            // hardware operations.
            unsafe { Mailbox::instance() }.ack_irq();
        }
    }

    #[task(binds = Ext0, priority = 0x10, shared = [i2c, v_target])]
    struct TuneM0;
    impl RticTask for TuneM0 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[0].stat, &mut rbuf));

            let m0_speed_now = u32::from_le_bytes(rbuf);
            // HACK: there's something wrong with lock closure args in mrtic. Need an extra
            // line of init here.
            let mut v_target: u32 = 0;
            self.shared().v_target.lock(|v| v_target = v[0]);
            let bytes: [u8; 2] = compute_control(v_target, m0_speed_now).to_le_bytes();
            self.shared()
                .i2c
                .lock(|i2c| i2c.write(I2C_ADDRS.motors[0].tune, &bytes));
        }
    }

    #[task(binds = Ext1, priority = 0x10, shared = [i2c, v_target])]
    struct TuneM1;
    impl RticTask for TuneM1 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[1].stat, &mut rbuf));

            let m1_speed_now = u32::from_le_bytes(rbuf);
            // HACK: there's something wrong with lock closure args in mrtic. Need an extra
            // line of init here.
            let mut v_target: u32 = 0;
            self.shared().v_target.lock(|v| v_target = v[1]);
            let bytes: [u8; 2] = compute_control(v_target, m1_speed_now).to_le_bytes();
            self.shared()
                .i2c
                .lock(|i2c| i2c.write(I2C_ADDRS.motors[1].tune, &bytes));
        }
    }

    #[task(binds = Ext2, priority = 0x10, shared = [i2c, v_target])]
    struct TuneM2;
    impl RticTask for TuneM2 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[2].stat, &mut rbuf));

            let m2_speed_now = u32::from_le_bytes(rbuf);
            // HACK: there's something wrong with lock closure args in mrtic. Need an extra
            // line of init here.
            let mut v_target: u32 = 0;
            self.shared().v_target.lock(|v| v_target = v[2]);
            let bytes: [u8; 2] = compute_control(v_target, m2_speed_now).to_le_bytes();
            self.shared()
                .i2c
                .lock(|i2c| i2c.write(I2C_ADDRS.motors[2].tune, &bytes));
        }
    }

    #[task(binds = Ext3, priority = 0x10, shared = [i2c, v_target])]
    struct TuneM3;
    impl RticTask for TuneM3 {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            let mut rbuf = [0; 4];
            self.shared()
                .i2c
                .lock(|i2c| i2c.read(I2C_ADDRS.motors[3].stat, &mut rbuf));

            let m3_speed_now = u32::from_le_bytes(rbuf);
            // HACK: there's something wrong with lock closure args in mrtic. Need an extra
            // line of init here.
            let mut v_target: u32 = 0;
            self.shared().v_target.lock(|v| v_target = v[3]);
            let bytes: [u8; 2] = compute_control(v_target, m3_speed_now).to_le_bytes();

            self.shared()
                .i2c
                .lock(|i2c| i2c.write(I2C_ADDRS.motors[3].tune, &bytes));
        }
    }

    #[inline]
    /// Compute tuning voltage to control motor power.
    fn compute_control(v_target: u32, speed_now: u32) -> i16 {
        // Resistance in mOhm
        let res = 10_000;
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
    /*
    #[cfg_attr(not(feature = "intc-edfic"), task(binds = MachineTimer = 0xff, priority=1, shared=[mtimer]))]
    #[cfg_attr(feature = "intc-edfic", task(binds = MachineTimer = 0x0, priority=1, shared=[mtimer]))]
    */
    #[task(binds = MachineTimer, priority = 0xff)]
    struct Finish;
    impl RticTask for Finish {
        fn init() -> Self {
            Self
        }
        fn exec(&mut self) {
            // HACK: exec moved to init task second run
        }
    }
    */
}
