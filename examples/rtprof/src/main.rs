#![no_main]
#![no_std]
#![allow(static_mut_refs)]

use bsp::rt as _;

#[cfg_attr(feature = "obs", rtic::app(device = bsp, obs = obs_trace::Obs /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
#[cfg_attr(not(feature = "obs"), rtic::app(device = bsp /*, dispatchers = [Timer0Ovf, Timer1Ovf, Timer2Ovf, Timer3Ovf, Ext0, Ext1, Ext2, Ext3]*/))]
mod app {
    use core::task;

    use bsp::{
        CPU_FREQ_HZ,
        apb_uart::ApbUart,
        cfg_regs::CfgRegs,
        clear_perf_counters,
        fugit::{ExtU32, ExtU64},
        i2c::{self, I2c},
        interrupt::Interrupt,
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
        timer_queue::TimerQueue,
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

    #[derive(Clone)]
    pub struct ImuPacket {
        timestamp: u64,
        ax: i32,
        ay: i32,
        az: i32,
        gx: i32,
        gy: i32,
        gz: i32,
    }

    const TASK_SET_SIZE: usize = 5;
    const I2C_ADDR_DURATION: u32 = 550;
    const I2C_DATA_DURATION: u32 = 500;

    const NUM_PARAMS: u8 = 6;
    const NUM_PARAM_BYTES: u8 = 4;
    const I2C_NUM_BYTES: u8 = NUM_PARAMS * NUM_PARAM_BYTES;

    const TASK_SET: [Task; TASK_SET_SIZE] = [
        Task::new(2000, 3, 3),
        Task::new(0xFFF, 4, 4),
        Task::new(0xFFF, 6, 6),
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
        i2c_byte_count: u8,
        imu_packet: ImuPacket,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        let i2c = I2c::init(100);
        let cfg = CfgRegs::init();
        let (_ibx, mut obx) = unsafe { Mailbox::instance() }.split();

        let imu_packet = ImuPacket {
            timestamp: (0),
            ax: (0),
            ay: (0),
            az: (0),
            gx: (0),
            gy: (0),
            gz: (0),
        };

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

        sprintln!("TODO: out of date, compute for updated tasks");

        /*
        for i in 0..TASK_SET_SIZE {
            sprintln!(
                "Task {i}: F (per HP): {}, Total runtime (us): {}",
                TASK_FREQ[i],
                TASK_RT[i]
            )
        } */

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

        timers[0].set_period(TASK_SET[0].period_us.micros());
        //timers[1].set_period(TASK_SET[3].period_us.micros());

        timers.iter_mut().for_each(Periodic::start);

        clear_perf_counters();
        rtprof_start_full();

        Shared {
            i2c,
            i2c_byte_count: 0,
            imu_packet,
        }
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
    struct I2cReq {}
    impl RticTask for I2cReq {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(0);

            const I2C_ADDR: u8 = 0x1;
            let mut last = false;

            self.shared().i2c.lock(|i2c| {
                i2c.wf_read_addr(I2C_ADDR);
            });

            TimerQueue::instance().push_rel(Interrupt::Timer0Ovf, I2C_ADDR_DURATION.nanos());
            rtprof_end_task(0);
        }
    }
    #[task(binds = Timer0Ovf, priority = 225, shared = [i2c, i2c_byte_count])]
    struct I2cCmd {}
    impl RticTask for I2cCmd {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(1);

            let mut last = false;

            let count = self.shared().i2c_byte_count.lock(|buf| *buf);

            if count == I2C_NUM_BYTES - 1 {
                last = true;
            }

            self.shared().i2c.lock(|i2c| {
                i2c.wf_read_cmd(last);
            });

            TimerQueue::instance().push_rel(Interrupt::Timer1Ovf, I2C_DATA_DURATION.nanos());
            rtprof_end_task(1);
        }
    }
    #[task(binds = Timer1Ovf, priority = 225, shared = [i2c, i2c_byte_count, imu_packet])]
    struct I2cRsp {
        data_word: u32,
    }
    impl RticTask for I2cRsp {
        fn init() -> Self {
            Self { data_word: 0 }
        }
        fn exec(&mut self) {
            rtprof_start_task(2);

            let mut rbuf: [u8; 1] = [0; 1];

            self.shared().i2c.lock(|i2c| {
                i2c.wf_read_rsp(&mut rbuf);
            });

            let count: u8 = self.shared().i2c_byte_count.lock(|buf| *buf);
            let byte_idx = count % 4;
            let word_idx = count / 4;

            self.data_word |= (rbuf[0] as u32) << byte_idx;

            if count < I2C_NUM_BYTES - 1 {
                if (byte_idx == 0) & (count != 0) {
                    match word_idx {
                        0 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.ax = self.data_word as i32),
                        1 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.ay = self.data_word as i32),
                        2 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.az = self.data_word as i32),
                        3 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.gx = self.data_word as i32),
                        4 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.gy = self.data_word as i32),
                        5 => self
                            .shared()
                            .imu_packet
                            .lock(|buf| buf.gz = self.data_word as i32),
                        _ => panic!("weird word idx"),
                    }

                    self.data_word = 0;
                }

                self.shared().i2c_byte_count.lock(|buf| *buf += 1);

                rtprof_end_task(2);
                // Repend I2cCmd until bytecount met
                TimerQueue::instance().push_now(Interrupt::Timer0Ovf);
            } else {
                self.shared().i2c_byte_count.lock(|buf| *buf = 0);
                // Capture timestamp once rest of packet received
                self.shared()
                    .imu_packet
                    .lock(|buf| buf.timestamp = MTimer::instance().now().as_micros());

                rtprof_end_task(2);
                TimerQueue::instance().push_now(Interrupt::Timer2Ovf);
            }
        }
    }

    // DL: 30 Prio: 255 - 30 = 225
    #[task(binds = Timer2Ovf, priority = 100, shared = [imu_packet])]
    struct FloatTask {}
    impl RticTask for FloatTask {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            rtprof_start_task(3);

            let packet: ImuPacket = self.shared().imu_packet.lock(|buf| buf.clone());

            sprintln!("{:X}", packet.timestamp);
            sprintln!("{:X}", packet.ax);
            sprintln!("{:X}", packet.ay);
            sprintln!("{:X}", packet.az);
            sprintln!("{:X}", packet.gx);
            sprintln!("{:X}", packet.gy);
            sprintln!("{:X}", packet.gz);

            /*
            let float1 = mmio::read_u32(0x20000) as f32;
            let float2 = mmio::read_u32(0x20020) as f32;

            sprintln!("{}", float1 - float2);
            */
            rtprof_end_task(3);
        }
    }

    /*
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
    } */
}
