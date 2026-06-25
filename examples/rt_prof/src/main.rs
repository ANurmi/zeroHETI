#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;
#[rtic::app(device = bsp, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3])]

mod app {
    // TODO: correct abstraction for addresses
    const SIM_START: u32 = 0x0100_0000;
    const SIM_STOP: u32 = 0x0100_0001;
    const SIM_PRESCALER: u32 = 0x0100_0002;
    const SIM_LOAD: u32 = 0x0100_0003;
    const SIM_SEED: u32 = 0x0100_0004;

    const TASK_MBX_PER: u32 = 0x0200_0000;
    const TASK_MBX_DLN: u32 = 0x0200_0001;
    const TASK_MBX_ACK: u32 = 0x0200_0002;

    const TASK_UPD_0_PER: u32 = 0x0201_0000;
    const TASK_UPD_0_DLN: u32 = 0x0201_0001;
    const TASK_UPD_0_ACK: u32 = 0x0201_0002;

    const TASK_UPD_1_PER: u32 = 0x0202_0000;
    const TASK_UPD_1_DLN: u32 = 0x0202_0001;
    const TASK_UPD_1_ACK: u32 = 0x0202_0002;

    const TASK_UPD_2_PER: u32 = 0x0203_0000;
    const TASK_UPD_2_DLN: u32 = 0x0203_0001;
    const TASK_UPD_2_ACK: u32 = 0x0203_0002;

    const TASK_UPD_3_PER: u32 = 0x0204_0000;
    const TASK_UPD_3_DLN: u32 = 0x0204_0001;
    const TASK_UPD_3_ACK: u32 = 0x0204_0002;

    const TASK_CTRL_0_PER: u32 = 0x0205_0000;
    const TASK_CTRL_0_DLN: u32 = 0x0205_0001;
    const TASK_CTRL_0_ACK: u32 = 0x0205_0002;

    const TASK_CTRL_1_PER: u32 = 0x0206_0000;
    const TASK_CTRL_1_DLN: u32 = 0x0206_0001;
    const TASK_CTRL_1_ACK: u32 = 0x0206_0002;

    const TASK_CTRL_2_PER: u32 = 0x0207_0000;
    const TASK_CTRL_2_DLN: u32 = 0x0207_0001;
    const TASK_CTRL_2_ACK: u32 = 0x0207_0002;

    const TASK_CTRL_3_PER: u32 = 0x0208_0000;
    const TASK_CTRL_3_DLN: u32 = 0x0208_0001;
    const TASK_CTRL_3_ACK: u32 = 0x0208_0002;

    const TASK_REP_0_PER: u32 = 0x0209_0000;
    const TASK_REP_0_DLN: u32 = 0x0209_0001;
    const TASK_REP_0_ACK: u32 = 0x0209_0002;

    const TASK_REP_1_PER: u32 = 0x020A_0000;
    const TASK_REP_1_DLN: u32 = 0x020A_0001;
    const TASK_REP_1_ACK: u32 = 0x020A_0002;

    const TASK_REP_2_PER: u32 = 0x020B_0000;
    const TASK_REP_2_DLN: u32 = 0x020B_0001;
    const TASK_REP_2_ACK: u32 = 0x020B_0002;

    const TASK_REP_3_PER: u32 = 0x020C_0000;
    const TASK_REP_3_DLN: u32 = 0x020C_0001;
    const TASK_REP_3_ACK: u32 = 0x020C_0002;

    /// Sending a letter to this address causes a print of certain format
    const MBX_PRINT_ADDR: u32 = 0x0300_0000;

    // Motor I2C addresses
    const I2C_M0_ADDR: u8 = 0x10;
    const I2C_M1_ADDR: u8 = 0x11;
    const I2C_M2_ADDR: u8 = 0x12;
    const I2C_M3_ADDR: u8 = 0x13;

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

    // rt_prof-specific

    #[repr(usize)]
    pub enum MotorIdx {
        M0 = 0,
        M1 = 1,
        M2 = 2,
        M3 = 3,
    }

    #[inline]
    fn sim_task_pend(obx: &mut Outbox, addr: u32) {
        obx.send(addr, 0x1);
    }

    #[inline]
    fn sim_task_ack(obx: &mut Outbox, addr: u32) {
        obx.send(addr, 0x0);
    }

    fn restart_timer_with_period(idx: MotorIdx, nperiod: timer_group::Duration) {
        let timer_addr = TIMER0_ADDR + (idx as usize) * TIMER_SEP;
        let mut timer = unsafe { Timer::instance_dyn(timer_addr) }.into_periodic();
        timer.cancel();
        timer.set_period(nperiod);
        timer.start();
    }

    /// Runtime in millis, read from env RUNTIME_MS
    const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;
    /// Load factor, read from env LOAD_FACTOR
    const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
    /// Control task initial period
    const CTRL_TASK_PER_US: u32 = 2000;

    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        i2c::{self, I2c},
        mailbox::{Inbox, Mailbox, Outbox},
        mmap::{
            apb_timer::{TIMER_SEP, TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
            mailbox::MBX_ADDR,
        },
        mtimer::{self, *},
        riscv::{self},
        sprintln,
        tb::signal_pass,
        timer_group::{self, Periodic, Timer},
    };
    use fugit::{ExtU32, ExtU64};

    #[derive(Clone, Copy, Default)]
    struct PidState {
        /// Integral
        int: i32,
        /// Previous error
        perr: i16,
    }

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
        ibx: Inbox,
        obx: Outbox,
        mail_buf_0: u32,
        mail_buf_1: u32,
        mail_buf_2: u32,
        mail_buf_3: u32,
        ctrl_buf_0: u8,
        ctrl_buf_1: u8,
        ctrl_buf_2: u8,
        ctrl_buf_3: u8,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        sprintln!("\n\r### Starting rt_prof (rtic) benchmark ###\n\r");
        let i2c = I2c::init(4);
        let (ibx, obx) = unsafe { Mailbox::instance::<{ MBX_ADDR }>() }.split();

        MTimer::instance().into_oneshot().start(100u64.micros());

        Shared {
            i2c,
            ibx,
            obx,
            mail_buf_0: 0,
            mail_buf_1: 0,
            mail_buf_2: 0,
            mail_buf_3: 0,
            ctrl_buf_0: 0,
            ctrl_buf_1: 0,
            ctrl_buf_2: 0,
            ctrl_buf_3: 0,
        }
    }

    #[task(binds = MachineTimer, priority = 0xff,
        /* shared = use raw pointers/instances instead, during init */
    )]
    struct StartSim {
        mtimer: mtimer::OneShot,
        start_time: Option<u64>,
    }

    impl RticTask for StartSim {
        fn init() -> Self {
            let mtimer = MTimer::instance().into_oneshot();
            Self {
                mtimer,
                start_time: None,
            }
        }
        fn exec(&mut self) {
            match self.start_time.as_mut() {
                None => {
                    sprintln!("MachineTimer::Setup");
                    // Start hyperperiod timer
                    self.start_time.replace(self.mtimer.counter());
                    self.mtimer.start(RUNTIME_MS.millis());
                    sprintln!("- Runtime         (ms): {}", RUNTIME_MS);
                    sprintln!("- Load factor  (0-100): {}", LF);

                    let timers = &mut [
                        Timer::init::<TIMER0_ADDR>().into_periodic(),
                        Timer::init::<TIMER1_ADDR>().into_periodic(),
                        Timer::init::<TIMER2_ADDR>().into_periodic(),
                        Timer::init::<TIMER3_ADDR>().into_periodic(),
                    ];

                    timers
                        .iter_mut()
                        .for_each(|t| t.set_period(CTRL_TASK_PER_US.micros()));

                    timers.iter_mut().for_each(Periodic::start);

                    // Safety: sim and app are not yet running
                    let (_, mut obx) = unsafe { Mailbox::instance::<{ MBX_ADDR }>() }.split();

                    obx.send_many(&[(SIM_PRESCALER, 10), (SIM_LOAD, LF)]);
                    obx.send(TASK_MBX_DLN, 0x400);
                    obx.send_many(&[
                        (TASK_REP_0_DLN, 0x400),
                        (TASK_REP_1_DLN, 0x400),
                        (TASK_REP_2_DLN, 0x400),
                        (TASK_REP_3_DLN, 0x400),
                    ]);
                    obx.send_many(&[
                        (TASK_UPD_0_DLN, 0x400),
                        (TASK_UPD_1_DLN, 0x400),
                        (TASK_UPD_2_DLN, 0x400),
                        (TASK_UPD_3_DLN, 0x400),
                    ]);
                    obx.send_many(&[
                        (TASK_CTRL_0_DLN, 0x400),
                        (TASK_CTRL_1_DLN, 0x400),
                        (TASK_CTRL_2_DLN, 0x400),
                        (TASK_CTRL_3_DLN, 0x400),
                    ]);
                    obx.send(SIM_START, 0x0);

                    unsafe {
                        // Clear instruction & cycle counters
                        riscv::register::minstret::write(0);
                        riscv::register::mcycle::write(0);
                        riscv::register::minstreth::write(0);
                        riscv::register::mcycleh::write(0);
                    }
                }
                Some(start_time) => {
                    let duration_real_cc = MTimer::instance().into_oneshot().duration().ticks();

                    // Terminate scoreboard
                    // Safety: unsure if safe. We'll do it anyway.
                    let (_, mut obx) = unsafe { Mailbox::instance::<{ MBX_ADDR }>() }.split();
                    obx.send(SIM_STOP, 0x1);

                    let instret = riscv::register::minstret::read64();
                    let active_time_cc = riscv::register::mcycle::read64();

                    sprintln!("MachineTimer::Teardown");

                    sprintln!("- Retired instructions: {instret}");
                    sprintln!("- Total time      (cc): {duration_real_cc}");
                    sprintln!("- active time     (cc): {active_time_cc}");
                    sprintln!(
                        "- CPU utilization  (%): {}",
                        (active_time_cc * 100) / duration_real_cc
                    );
                    signal_pass(None);
                }
            }
        }
    }

    fn compute_pid(input: u8, s_pid: &mut PidState, ctrl_buf: &mut u8) -> u8 {
        // Discrete-time PID controller (per-call integral/derivative coefficients)
        const SETPOINT: i16 = 127;
        const KP: i32 = 1; // proportional gain
        const KI: i32 = 1; // integral gain (per step)
        const KD: i32 = 1; // derivative gain (per step)
        const INTEGRAL_MAX: i32 = 10_000;

        let err: i16 = SETPOINT - input as i16;

        // Accumulate integral (anti-windup via clamping)
        s_pid.int = (s_pid.int + err as i32).clamp(-INTEGRAL_MAX, INTEGRAL_MAX);

        let deriv: i32 = (err - s_pid.perr) as i32;

        let p_term: i32 = KP * (err as i32);
        let i_term: i32 = KI * s_pid.int;
        let d_term: i32 = KD * deriv;

        s_pid.perr = err;

        let mut out: i32 = SETPOINT as i32 + (p_term + i_term + d_term);
        out = out.clamp(0, 255);

        // used by report tasks
        *ctrl_buf = input as u8;

        out as u8
    }

    // Hardware tasks
    #[task(binds = Timer0Cmp, priority = 0x88, shared = [i2c, ctrl_buf_0, obx])]
    struct Ctrl0 {
        state: PidState,
    }
    impl RticTask for Ctrl0 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control0]");

            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2C_M0_ADDR, &mut rbuf);
            });
            let measured_v = u8::from_le_bytes(rbuf);

            let out_v = self
                .shared()
                .ctrl_buf_0
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2C_M0_ADDR, &[out_v]);
            });

            self.shared().obx.lock(|obx| {
                sim_task_pend(obx, TASK_REP_0_ACK);
                Report0::spawn(()).unwrap();
                sim_task_ack(obx, TASK_CTRL_0_ACK);
            });
        }
    }

    #[task(binds = Timer1Cmp, priority = 0x88, shared = [i2c, ctrl_buf_1, obx])]
    struct Ctrl1 {
        state: PidState,
    }
    impl RticTask for Ctrl1 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control1]");

            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2C_M1_ADDR, &mut rbuf);
            });
            let measured_v = u8::from_le_bytes(rbuf);

            let out_v = self
                .shared()
                .ctrl_buf_1
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2C_M1_ADDR, &[out_v]);
            });

            self.shared().obx.lock(|obx| {
                sim_task_pend(obx, TASK_REP_1_ACK);
                Report1::spawn(()).unwrap();
                sim_task_ack(obx, TASK_CTRL_1_ACK);
            });
        }
    }

    #[task(binds = Timer2Cmp, priority = 0x88, shared = [i2c, ctrl_buf_2, obx])]
    struct Ctrl2 {
        state: PidState,
    }
    impl RticTask for Ctrl2 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control2]");

            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2C_M2_ADDR, &mut rbuf);
            });
            let measured_v = u8::from_le_bytes(rbuf);

            let out_v = self
                .shared()
                .ctrl_buf_2
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2C_M2_ADDR, &[out_v]);
            });

            self.shared().obx.lock(|obx| {
                sim_task_pend(obx, TASK_REP_2_ACK);
                Report2::spawn(()).unwrap();
                sim_task_ack(obx, TASK_CTRL_2_ACK);
            })
        }
    }

    #[task(binds = Timer3Cmp, priority = 0x88, shared = [i2c, ctrl_buf_3, obx])]
    struct Ctrl3 {
        state: PidState,
    }
    impl RticTask for Ctrl3 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control3]");

            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2C_M3_ADDR, &mut rbuf);
            });
            let measured_v = u8::from_le_bytes(rbuf);

            let out_v = self
                .shared()
                .ctrl_buf_3
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2C_M3_ADDR, &[out_v]);
            });

            self.shared().obx.lock(|obx| {
                sim_task_pend(obx, TASK_REP_3_ACK);
                Report3::spawn(()).unwrap();
                sim_task_ack(obx, TASK_CTRL_3_ACK);
            });
        }
    }

    #[task(binds = Mbx, priority = 0xf1, shared = [mail_buf_0, mail_buf_1, mail_buf_2, mail_buf_3, ibx, obx])]
    struct Mail {}
    impl RticTask for Mail {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            let mut letters = [(0, 0); 4];
            self.shared().ibx.lock(|ibx| {
                ibx.recv_many(&mut letters);
            });

            for (addr, data) in letters {
                match addr {
                    0x100 => self.shared().mail_buf_0.lock(|mail| *mail = data),
                    0x101 => self.shared().mail_buf_1.lock(|mail| *mail = data),
                    0x102 => self.shared().mail_buf_2.lock(|mail| *mail = data),
                    0x103 => self.shared().mail_buf_3.lock(|mail| *mail = data),
                    _ => sprintln!("Weird letter"),
                }
            }
            sprintln!("[Mailbox]");
            self.shared().obx.lock(|obx| {
                sim_task_pend(obx, TASK_UPD_0_ACK);
                Update0::spawn(()).unwrap();
                sim_task_pend(obx, TASK_UPD_1_ACK);
                Update1::spawn(()).unwrap();
                sim_task_pend(obx, TASK_UPD_2_ACK);
                Update2::spawn(()).unwrap();
                sim_task_pend(obx, TASK_UPD_3_ACK);
                Update3::spawn(()).unwrap();
                sim_task_ack(obx, TASK_MBX_ACK);
            });
        }
    }

    // Software tasks
    #[sw_task(priority = 0x11, shared = [mail_buf_0, obx])]
    struct Update0;
    impl RticSwTask for Update0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 0]");

            let nperiod_us = self.shared().mail_buf_0.lock(|m| *m);
            restart_timer_with_period(MotorIdx::M0, nperiod_us.micros());
            self.shared().obx.lock(|obx| {
                obx.send(TASK_CTRL_0_PER, nperiod_us);
                obx.send(TASK_CTRL_0_DLN, nperiod_us);

                // invalidate these if currently active
                sim_task_ack(obx, TASK_CTRL_0_ACK);
                sim_task_ack(obx, TASK_REP_0_ACK);
                sim_task_ack(obx, TASK_UPD_0_ACK);
            });
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_1, obx])]
    struct Update1;
    impl RticSwTask for Update1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 1]");

            self.shared().obx.lock(|obx| {
                let nperiod_us = self.shared().mail_buf_1.lock(|m| *m);
                restart_timer_with_period(MotorIdx::M1, nperiod_us.micros());
                obx.send(TASK_CTRL_1_PER, nperiod_us);
                obx.send(TASK_CTRL_1_DLN, nperiod_us);
                // invalidate these if currently active
                sim_task_ack(obx, TASK_CTRL_1_ACK);
                sim_task_ack(obx, TASK_REP_1_ACK);
                sim_task_ack(obx, TASK_UPD_1_ACK);
            });
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_2, obx])]
    struct Update2;
    impl RticSwTask for Update2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 2]");

            // invalidate these if currently active
            self.shared().obx.lock(|obx| {
                let nperiod_us: u32 = self.shared().mail_buf_2.lock(|m| *m);
                restart_timer_with_period(MotorIdx::M2, nperiod_us.micros());
                obx.send(TASK_CTRL_2_PER, nperiod_us);
                obx.send(TASK_CTRL_2_DLN, nperiod_us);

                sim_task_ack(obx, TASK_CTRL_2_ACK);
                sim_task_ack(obx, TASK_REP_2_ACK);
                sim_task_ack(obx, TASK_UPD_2_ACK);
            });
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_3, obx])]
    struct Update3;
    impl RticSwTask for Update3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 3]");

            // invalidate these if currently active
            self.shared().obx.lock(|obx| {
                let nperiod_us: u32 = self.shared().mail_buf_3.lock(|m| *m);
                restart_timer_with_period(MotorIdx::M3, nperiod_us.micros());
                obx.send(TASK_CTRL_3_PER, nperiod_us);
                obx.send(TASK_CTRL_3_DLN, nperiod_us);

                sim_task_ack(obx, TASK_CTRL_3_ACK);
                sim_task_ack(obx, TASK_REP_3_ACK);
                sim_task_ack(obx, TASK_UPD_3_ACK);
            });
        }
    }

    #[sw_task(priority = 0x10, shared = [ctrl_buf_0, obx])]
    struct Report0;
    impl RticSwTask for Report0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 0]");

            let time_now = MTimer::instance().into_oneshot().duration().to_micros();
            let ctrl_buf = self.shared().ctrl_buf_0.lock(|buf| *buf);
            let rep_letter = ((time_now as u32) << 16) | ((0u8 as u32) << 8) | (ctrl_buf as u32);
            self.shared().obx.lock(|obx| {
                obx.send(MBX_PRINT_ADDR, rep_letter);

                sim_task_ack(obx, TASK_REP_0_ACK);
            });
        }
    }

    #[sw_task(priority = 0x10, shared = [ctrl_buf_1, obx])]
    struct Report1;
    impl RticSwTask for Report1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 1]");

            let time_now = MTimer::instance().into_oneshot().duration().to_micros();
            let ctrl_buf = self.shared().ctrl_buf_1.lock(|buf| *buf);
            let rep_letter = ((time_now as u32) << 16) | ((1u8 as u32) << 8) | (ctrl_buf as u32);
            self.shared().obx.lock(|obx| {
                obx.send(MBX_PRINT_ADDR, rep_letter);

                sim_task_ack(obx, TASK_REP_1_ACK);
            });
        }
    }
    #[sw_task(priority = 0x10, shared = [ctrl_buf_2, obx])]
    struct Report2;
    impl RticSwTask for Report2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 2]");

            let time_now = MTimer::instance().into_oneshot().duration().to_micros();
            let ctrl_buf = self.shared().ctrl_buf_2.lock(|buf| *buf);
            let rep_letter = ((time_now as u32) << 16) | ((2u8 as u32) << 8) | (ctrl_buf as u32);
            self.shared().obx.lock(|obx| {
                obx.send(MBX_PRINT_ADDR, rep_letter);

                sim_task_ack(obx, TASK_REP_2_ACK);
            });
        }
    }
    #[sw_task(priority = 0x10, shared = [ctrl_buf_3, obx])]
    struct Report3;
    impl RticSwTask for Report3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 3]");

            let time_now = MTimer::instance().into_oneshot().duration().to_micros();
            let ctrl_buf = self.shared().ctrl_buf_3.lock(|buf| *buf);
            let rep_letter = ((time_now as u32) << 16) | ((3u8 as u32) << 8) | (ctrl_buf as u32);
            self.shared().obx.lock(|obx| {
                obx.send(MBX_PRINT_ADDR, rep_letter);

                sim_task_ack(obx, TASK_REP_3_ACK);
            });
        }
    }
}
