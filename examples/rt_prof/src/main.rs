#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;
#[rtic::app(device = bsp, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3])]

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

    const fn to_prio(per: u32) -> u8 {
        // Shifting out lowest 8 bits makes timing granularity
        // 256 (us), which is sufficient to represent the set
        // of deadlines in this application.
        (255 - (per >> 8)) as u8
    }

    // rt_prof-specific

    #[derive(Clone, Copy)]
    #[repr(u32)]
    #[allow(unused)]
    enum MbxAddr {
        SimStart = 0x100_0000,
        SimStop = 0x100_0001,
        SimPrescaler = 0x100_0002,
        SimLoad = 0x100_0003,
        SimSeed = 0x100_0004,

        /// Period
        TaskMbxPer = 0x200_0000,
        /// Deadline
        TaskMbxDln = 0x200_0001,
        /// Acknowledge
        TaskMbxAck = 0x200_0002,

        TaskUpd0Per = 0x201_0000,
        TaskUpd0Dln = 0x201_0001,
        TaskUpd0Ack = 0x201_0002,

        TaskUpd1Per = 0x202_0000,
        TaskUpd1Dln = 0x202_0001,
        TaskUpd1Ack = 0x202_0002,

        TaskUpd2Per = 0x203_0000,
        TaskUpd2Dln = 0x203_0001,
        TaskUpd2Ack = 0x203_0002,

        TaskUpd3Per = 0x204_0000,
        TaskUpd3Dln = 0x204_0001,
        TaskUpd3Ack = 0x204_0002,

        TaskCtrl0Per = 0x205_0000,
        TaskCtrl0Dln = 0x205_0001,
        TaskCtrl0Ack = 0x205_0002,

        TaskCtrl1Per = 0x206_0000,
        TaskCtrl1Dln = 0x206_0001,
        TaskCtrl1Ack = 0x206_0002,

        TaskCtrl2Per = 0x207_0000,
        TaskCtrl2Dln = 0x207_0001,
        TaskCtrl2Ack = 0x207_0002,

        TaskCtrl3Per = 0x208_0000,
        TaskCtrl3Dln = 0x208_0001,
        TaskCtrl3Ack = 0x208_0002,

        TaskRep0Per = 0x209_0000,
        TaskRep0Dln = 0x209_0001,
        TaskRep0Ack = 0x209_0002,

        TaskRep1Per = 0x20A_0000,
        TaskRep1Dln = 0x20A_0001,
        TaskRep1Ack = 0x20A_0002,

        TaskRep2Per = 0x20B_0000,
        TaskRep2Dln = 0x20B_0001,
        TaskRep2Ack = 0x20B_0002,

        TaskRep3Per = 0x20C_0000,
        TaskRep3Dln = 0x20C_0001,
        TaskRep3Ack = 0x20C_0002,

        /// Sending a letter to this address causes a print of certain format
        Print = 0x300_0000,
    }

    // Motor I2C addresses
    #[derive(Clone, Copy)]
    #[repr(u8)]
    enum I2cAddr {
        M0 = 0x10,
        M1 = 0x11,
        M2 = 0x12,
        M3 = 0x13,
    }

    #[repr(usize)]
    pub enum MotorIdx {
        M0 = 0,
        M1 = 1,
        M2 = 2,
        M3 = 3,
    }

    #[inline]
    fn sim_task_pend(obx: &mut Outbox, addr: MbxAddr) {
        obx.send(addr as u32, 0x1);
    }

    #[inline]
    fn sim_task_ack(obx: &mut Outbox, addr: MbxAddr) {
        obx.send(addr as u32, 0x0);
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

    // Initiation inteval for report tasks, i.e., on every Nth
    // job of the matching control task a report job is spawned.
    const REP_TASK_II: u32 = 5;

    const MBX_TASK_DL_US: u32 = 1000;
    const UPD_TASK_DL_US: u32 = 2000;

    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        clic::Clic,
        i2c::{self, I2c},
        interrupt::Interrupt,
        mailbox::{Inbox, Mailbox, Outbox},
        mmap::apb_timer::{TIMER_SEP, TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mtimer::{self, *},
        register::Permission::W,
        riscv::{self},
        sprintln,
        tb::signal_pass,
        timer_group::{self, Periodic, Timer},
    };
    use core::fmt::Write;
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
        serial: ApbUart,
        i2c: i2c::I2c,
        ibx: Inbox,
        obx: Outbox,
        mail_buf_0: u32,
        mail_buf_1: u32,
        mail_buf_2: u32,
        mail_buf_3: u32,
        read_buf_0: u8,
        read_buf_1: u8,
        read_buf_2: u8,
        read_buf_3: u8,
        ctrl_buf_0: u8,
        ctrl_buf_1: u8,
        ctrl_buf_2: u8,
        ctrl_buf_3: u8,
    }

    #[init]
    fn init() -> Shared {
        let serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        sprintln!("\n\r### Starting rt_prof (rtic) benchmark ###\n\r");
        let i2c = I2c::init(4);
        let (ibx, obx) = unsafe { Mailbox::instance() }.split();

        MTimer::instance().into_oneshot().start(100u64.micros());

        Shared {
            serial,
            i2c,
            ibx,
            obx,
            mail_buf_0: 0,
            mail_buf_1: 0,
            mail_buf_2: 0,
            mail_buf_3: 0,
            read_buf_0: 0,
            read_buf_1: 0,
            read_buf_2: 0,
            read_buf_3: 0,
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
                    let (_, mut obx) = unsafe { Mailbox::instance() }.split();

                    const CMDS: &[(MbxAddr, u32)] = &[
                        (MbxAddr::SimPrescaler, 10),
                        (MbxAddr::SimLoad, LF),
                        (MbxAddr::TaskMbxDln, MBX_TASK_DL_US),
                        // Initialize with very long deadlines
                        (MbxAddr::TaskRep0Dln, 0x1000),
                        (MbxAddr::TaskRep1Dln, 0x1000),
                        (MbxAddr::TaskRep2Dln, 0x1000),
                        (MbxAddr::TaskRep3Dln, 0x1000),
                        // Initialize with very long deadlines
                        (MbxAddr::TaskUpd0Dln, UPD_TASK_DL_US),
                        (MbxAddr::TaskUpd1Dln, UPD_TASK_DL_US),
                        (MbxAddr::TaskUpd2Dln, UPD_TASK_DL_US),
                        (MbxAddr::TaskUpd3Dln, UPD_TASK_DL_US),
                        // Initialize with very long deadlines
                        (MbxAddr::TaskCtrl0Dln, 0x1000),
                        (MbxAddr::TaskCtrl1Dln, 0x1000),
                        (MbxAddr::TaskCtrl2Dln, 0x1000),
                        (MbxAddr::TaskCtrl3Dln, 0x1000),
                        (MbxAddr::SimStart, 0x0),
                    ];
                    CMDS.iter()
                        .for_each(|&(addr, data)| obx.send(addr as u32, data));

                    unsafe {
                        // Clear instruction & cycle counters
                        riscv::register::minstret::write(0);
                        riscv::register::mcycle::write(0);
                        riscv::register::minstreth::write(0);
                        riscv::register::mcycleh::write(0);
                    }
                }
                Some(start_time) => {
                    // Take timestamp for sim end
                    let now = MTimer::instance().into_oneshot().duration().ticks();

                    // Terminate scoreboard
                    // Safety: unsure if safe. We'll do it anyway.
                    let (_, mut obx) = unsafe { Mailbox::instance() }.split();
                    obx.send(MbxAddr::SimStop as u32, 0x1);

                    // Calculate simulation duration, accounting for setup time
                    let duration_real_cc = now - *start_time;
                    let setup_time_real_cc = *start_time;

                    let instret = riscv::register::minstret::read64();
                    let active_time_cc = riscv::register::mcycle::read64();

                    sprintln!("MachineTimer::Teardown");

                    sprintln!("- Retired instructions:      {instret}");
                    sprintln!("- Total time w/o setup (cc): {duration_real_cc}");
                    sprintln!("  * Setup              (cc): {setup_time_real_cc}");
                    sprintln!("  * Active time        (cc): {active_time_cc}");
                    sprintln!(
                        "- CPU utilization       (%): {}",
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

        // Some arbitrary float arithmetic to increase the computational intensity
        let mut out: i32 = SETPOINT as i32 + (p_term as f32 + i_term as f32 + d_term as f32) as i32;
        out = out.clamp(0, 255);

        // used by report tasks
        *ctrl_buf = out as u8;

        out as u8
    }

    // Hardware tasks
    #[task(binds = Timer0Cmp, priority = 0xfc, shared = [i2c, read_buf_0, obx, ctrl_buf_0])]
    struct Ctrl0 {
        state: PidState,
        rep_cnt: u32,
    }
    impl RticTask for Ctrl0 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
                rep_cnt: 3,
            }
        }
        fn exec(&mut self) {
            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2cAddr::M0 as u8, &mut rbuf);
            });

            let measured_v = u8::from_le_bytes(rbuf);
            self.shared().read_buf_0.lock(|buf| *buf = measured_v);

            let out_v = self
                .shared()
                .ctrl_buf_0
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2cAddr::M0 as u8, &[out_v]);
            });

            if self.rep_cnt >= REP_TASK_II - 1 {
                self.rep_cnt = 0;
                self.shared()
                    .obx
                    .lock(|obx| sim_task_pend(obx, MbxAddr::TaskRep0Ack));
                Report0::spawn(()).unwrap();
            } else {
                self.rep_cnt += 1;
            }
            self.shared()
                .obx
                .lock(|obx| sim_task_ack(obx, MbxAddr::TaskCtrl0Ack));
        }
    }

    #[task(binds = Timer1Cmp, priority = 0xfc, shared = [i2c,read_buf_1, obx, ctrl_buf_1])]
    struct Ctrl1 {
        state: PidState,
        rep_cnt: u32,
    }
    impl RticTask for Ctrl1 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
                rep_cnt: 5,
            }
        }
        fn exec(&mut self) {
            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2cAddr::M1 as u8, &mut rbuf);
            });

            let measured_v = u8::from_le_bytes(rbuf);
            self.shared().read_buf_1.lock(|buf| *buf = measured_v);

            let out_v = self
                .shared()
                .ctrl_buf_1
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2cAddr::M1 as u8, &[out_v]);
            });

            if self.rep_cnt >= REP_TASK_II - 1 {
                self.rep_cnt = 0;
                self.shared()
                    .obx
                    .lock(|obx| sim_task_pend(obx, MbxAddr::TaskRep1Ack));
                Report1::spawn(()).unwrap();
            } else {
                self.rep_cnt += 1;
            }
            self.shared()
                .obx
                .lock(|obx| sim_task_ack(obx, MbxAddr::TaskCtrl1Ack));
        }
    }

    #[task(binds = Timer2Cmp, priority = 0xfc, shared = [i2c, read_buf_2, obx, ctrl_buf_2])]
    struct Ctrl2 {
        state: PidState,
        rep_cnt: u32,
    }
    impl RticTask for Ctrl2 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
                rep_cnt: 1,
            }
        }
        fn exec(&mut self) {
            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2cAddr::M2 as u8, &mut rbuf);
            });

            let measured_v = u8::from_le_bytes(rbuf);
            self.shared().read_buf_2.lock(|buf| *buf = measured_v);

            let out_v = self
                .shared()
                .ctrl_buf_2
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2cAddr::M2 as u8, &[out_v]);
            });

            if self.rep_cnt >= REP_TASK_II - 1 {
                self.rep_cnt = 0;
                self.shared()
                    .obx
                    .lock(|obx| sim_task_pend(obx, MbxAddr::TaskRep2Ack));
                Report2::spawn(()).unwrap();
            } else {
                self.rep_cnt += 1;
            }
            self.shared()
                .obx
                .lock(|obx| sim_task_ack(obx, MbxAddr::TaskCtrl2Ack));
        }
    }

    #[task(binds = Timer3Cmp, priority = 0xfc, shared = [i2c, read_buf_3, obx, ctrl_buf_3])]
    struct Ctrl3 {
        state: PidState,
        rep_cnt: u32,
    }
    impl RticTask for Ctrl3 {
        fn init() -> Self {
            Self {
                state: PidState::default(),
                rep_cnt: 2,
            }
        }
        fn exec(&mut self) {
            let mut rbuf = [0];
            self.shared().i2c.lock(|i2c| {
                // Read motor status
                i2c.read(I2cAddr::M3 as u8, &mut rbuf);
            });

            let measured_v = u8::from_le_bytes(rbuf);
            self.shared().read_buf_3.lock(|buf| *buf = measured_v);

            let out_v = self
                .shared()
                .ctrl_buf_3
                .lock(|buf| compute_pid(measured_v, &mut self.state, buf));

            self.shared().i2c.lock(|i2c| {
                // Write to motor
                i2c.write(I2cAddr::M3 as u8, &[out_v]);
            });

            if self.rep_cnt >= REP_TASK_II - 1 {
                self.rep_cnt = 0;
                self.shared()
                    .obx
                    .lock(|obx| sim_task_pend(obx, MbxAddr::TaskRep3Ack));
                Report3::spawn(()).unwrap();
            } else {
                self.rep_cnt += 1;
            }
            self.shared()
                .obx
                .lock(|obx| sim_task_ack(obx, MbxAddr::TaskCtrl3Ack));
        }
    }

    #[task(binds = Mbx, priority = 0xfe, shared = [mail_buf_0, mail_buf_1, mail_buf_2, mail_buf_3, ibx, obx ])]
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
                    0x0 =>
                        /* ignore zero-addressed letters */
                        {}
                    0x100 => {
                        self.shared().mail_buf_0.lock(|mail| *mail = data);
                        self.shared()
                            .obx
                            .lock(|obx| sim_task_pend(obx, MbxAddr::TaskUpd0Ack));
                        Update0::spawn(()).unwrap();
                    }
                    0x101 => {
                        self.shared().mail_buf_1.lock(|mail| *mail = data);
                        self.shared()
                            .obx
                            .lock(|obx| sim_task_pend(obx, MbxAddr::TaskUpd1Ack));
                        Update1::spawn(()).unwrap();
                    }
                    0x102 => {
                        self.shared().mail_buf_2.lock(|mail| *mail = data);
                        self.shared()
                            .obx
                            .lock(|obx| sim_task_pend(obx, MbxAddr::TaskUpd2Ack));
                        Update2::spawn(()).unwrap();
                    }
                    0x103 => {
                        self.shared().mail_buf_3.lock(|mail| *mail = data);
                        self.shared()
                            .obx
                            .lock(|obx| sim_task_pend(obx, MbxAddr::TaskUpd3Ack));
                        Update3::spawn(()).unwrap();
                    }
                    _ => panic!("Weird letter")
                };
            }

            self.shared()
                .obx
                .lock(|obx| sim_task_ack(obx, MbxAddr::TaskMbxAck));
        }
    }

    // Software tasks
    #[sw_task(priority = 0xfb, shared = [mail_buf_0, obx])]
    struct Update0;
    impl RticSwTask for Update0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let nperiod_us = self.shared().mail_buf_0.lock(|m| *m);
            restart_timer_with_period(MotorIdx::M0, nperiod_us.micros());
            self.shared().obx.lock(|obx| {
                obx.send(MbxAddr::TaskCtrl0Per as u32, nperiod_us);
                obx.send(MbxAddr::TaskCtrl0Dln as u32, nperiod_us);
                obx.send(MbxAddr::TaskRep0Dln as u32, REP_TASK_II * nperiod_us);
                Clic::ctl(Interrupt::Timer0Cmp).set_level(to_prio(nperiod_us));
                // invalidate these if currently active
                sim_task_ack(obx, MbxAddr::TaskCtrl0Ack);
                sim_task_ack(obx, MbxAddr::TaskRep0Ack);
                sim_task_ack(obx, MbxAddr::TaskUpd0Ack);
            });
        }
    }

    #[sw_task(priority = 0xfb, shared = [mail_buf_1, obx])]
    struct Update1;
    impl RticSwTask for Update1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let nperiod_us = self.shared().mail_buf_1.lock(|m| *m);
            restart_timer_with_period(MotorIdx::M1, nperiod_us.micros());
            self.shared().obx.lock(|obx| {
                obx.send(MbxAddr::TaskCtrl1Per as u32, nperiod_us);
                obx.send(MbxAddr::TaskCtrl1Dln as u32, nperiod_us);
                obx.send(MbxAddr::TaskRep1Dln as u32, REP_TASK_II * nperiod_us);
                Clic::ctl(Interrupt::Timer1Cmp).set_level(to_prio(nperiod_us));
                // invalidate these if currently active
                sim_task_ack(obx, MbxAddr::TaskCtrl1Ack);
                sim_task_ack(obx, MbxAddr::TaskRep1Ack);
                sim_task_ack(obx, MbxAddr::TaskUpd1Ack);
            });
        }
    }

    #[sw_task(priority = 0xfb, shared = [mail_buf_2, obx])]
    struct Update2;
    impl RticSwTask for Update2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let nperiod_us: u32 = self.shared().mail_buf_2.lock(|m| *m);
            restart_timer_with_period(MotorIdx::M2, nperiod_us.micros());
            self.shared().obx.lock(|obx| {
                obx.send(MbxAddr::TaskCtrl2Per as u32, nperiod_us);
                obx.send(MbxAddr::TaskCtrl2Dln as u32, nperiod_us);
                obx.send(MbxAddr::TaskRep2Dln as u32, REP_TASK_II * nperiod_us);
                Clic::ctl(Interrupt::Timer2Cmp).set_level(to_prio(nperiod_us));
                sim_task_ack(obx, MbxAddr::TaskCtrl2Ack);
                sim_task_ack(obx, MbxAddr::TaskRep2Ack);
                sim_task_ack(obx, MbxAddr::TaskUpd2Ack);
            });
        }
    }

    #[sw_task(priority = 0xfb, shared = [mail_buf_3, obx])]
    struct Update3;
    impl RticSwTask for Update3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let nperiod_us: u32 = self.shared().mail_buf_3.lock(|m| *m);
            restart_timer_with_period(MotorIdx::M3, nperiod_us.micros());
            self.shared().obx.lock(|obx| {
                obx.send(MbxAddr::TaskCtrl3Per as u32, nperiod_us);
                obx.send(MbxAddr::TaskCtrl3Dln as u32, nperiod_us);
                obx.send(MbxAddr::TaskRep3Dln as u32, REP_TASK_II * nperiod_us);
                Clic::ctl(Interrupt::Timer3Cmp).set_level(to_prio(nperiod_us));
                sim_task_ack(obx, MbxAddr::TaskCtrl3Ack);
                sim_task_ack(obx, MbxAddr::TaskRep3Ack);
                sim_task_ack(obx, MbxAddr::TaskUpd3Ack);
            });
        }
    }

    #[sw_task(priority = 0xF1, shared = [ctrl_buf_0, read_buf_0, obx, serial])]
    struct Report0;
    impl RticSwTask for Report0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let ctrl_buf = self.shared().ctrl_buf_0.lock(|buf| *buf);
            let read_buf = self.shared().read_buf_0.lock(|buf| *buf);
            let task_idx: u8 = 0;

            let mut time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] measured    value {:3} from motor {:1} on I2C bus",
                    task_idx, time_now, read_buf, task_idx
                )
                .ok()
            });
            time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] set control value {:3} to   motor {:1} on I2C bus",
                    task_idx, time_now, ctrl_buf, task_idx
                )
                .ok()
            });

            self.shared().obx.lock(|obx| {
                // obx.send(MbxAddr::Print as u32, rep_letter);
                sim_task_ack(obx, MbxAddr::TaskRep0Ack);
            });
        }
    }

    #[sw_task(priority = 0xF1, shared = [ctrl_buf_1, read_buf_1, obx, serial])]
    struct Report1;
    impl RticSwTask for Report1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let ctrl_buf = self.shared().ctrl_buf_1.lock(|buf| *buf);
            let read_buf = self.shared().read_buf_1.lock(|buf| *buf);
            let task_idx: u8 = 1;

            let mut time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] measured    value {:3} from motor {:1} on I2C bus",
                    task_idx, time_now, read_buf, task_idx
                )
                .ok()
            });

            time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] set control value {:3} to   motor {:1} on I2C bus",
                    task_idx, time_now, ctrl_buf, task_idx
                )
                .ok()
            });
            self.shared().obx.lock(|obx| {
                // obx.send(MbxAddr::Print as u32, rep_letter);
                sim_task_ack(obx, MbxAddr::TaskRep1Ack);
            });
        }
    }
    #[sw_task(priority = 0xF1, shared = [ctrl_buf_2, read_buf_2, obx, serial])]
    struct Report2;
    impl RticSwTask for Report2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let ctrl_buf = self.shared().ctrl_buf_2.lock(|buf| *buf);
            let read_buf = self.shared().read_buf_2.lock(|buf| *buf);
            let task_idx: u8 = 2;

            let mut time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] measured    value {:3} from motor {:1} on I2C bus",
                    task_idx, time_now, read_buf, task_idx
                )
                .ok()
            });
            time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] set control value {:3} to   motor {:1} on I2C bus",
                    task_idx, time_now, ctrl_buf, task_idx
                )
                .ok()
            });

            self.shared().obx.lock(|obx| {
                // obx.send(MbxAddr::Print as u32, rep_letter);
                sim_task_ack(obx, MbxAddr::TaskRep2Ack);
            });
        }
    }
    #[sw_task(priority = 0xF1, shared = [ctrl_buf_3, read_buf_3, obx, serial])]
    struct Report3;
    impl RticSwTask for Report3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            let ctrl_buf = self.shared().ctrl_buf_3.lock(|buf| *buf);
            let read_buf = self.shared().read_buf_3.lock(|buf| *buf);
            let task_idx: u8 = 3;

            let mut time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] measured    value {:3} from motor {:1} on I2C bus",
                    task_idx, time_now, read_buf, task_idx
                )
                .ok()
            });

            time_now = MTimer::instance().into_oneshot().duration().to_micros();
            self.shared().serial.lock(|s| {
                writeln!(
                    s,
                    "[TaskRep-{} @{:6} us] set control value {:3} to   motor {:1} on I2C bus",
                    task_idx, time_now, ctrl_buf, task_idx
                )
                .ok()
            });

            self.shared().obx.lock(|obx| {
                // obx.send(MbxAddr::Print as u32, rep_letter);
                sim_task_ack(obx, MbxAddr::TaskRep3Ack);
            });
        }
    }
}
