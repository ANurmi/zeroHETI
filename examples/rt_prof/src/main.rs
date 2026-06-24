#![no_main]
#![no_std]
#![allow(static_mut_refs)]

mod mailbox;

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

    // TODO: move this to Mbx HAL
    #[inline]
    fn send_letter(addr: u32, data: u32) {
        mmio::write_u32(MAILBOX.ob.addr as usize, addr);
        mmio::write_u32(MAILBOX.ob.data as usize, data);
        // send letter
        mmio::write_u32(MAILBOX.ctrl as usize, 0x1);
    }
    #[inline]
    fn wait_outbox_empty() {
        while (mmio::read_u32(MAILBOX.stat as usize) & (0b1 << 2)) == 0 {}
    }

    // rt_prof-specific
    #[inline]
    fn pend_task(addr: u32) {
        send_letter(addr, 0x1);
    }

    #[inline]
    fn ack_task(addr: u32) {
        send_letter(addr, 0x0);
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
        core_interrupt,
        i2c::{self, I2c},
        interrupt::Interrupt::{self},
        mailbox::MotorIdx,
        mmap::apb_timer::{TIMER_SEP, TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
        mmio,
        mtimer::{self, *},
        riscv::{
            self,
            asm::{nop, wfi},
        },
        rt::entry,
        sprintln,
        tb::signal_pass,
        timer_group::{self, Periodic, Timer},
    };
    use fugit::{ExtU32, ExtU64};
    use riscv_rt::InterruptNumber;

    use crate::mailbox::MAILBOX;

    #[shared]
    struct Shared {
        i2c: i2c::I2c,
        mail_buf_0: u32,
        mail_buf_1: u32,
        mail_buf_2: u32,
        mail_buf_3: u32,
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        sprintln!("\n\r### Starting rt_prof (rtic) benchmark ###\n\r");
        let i2c = I2c::init(4);

        MTimer::instance().into_oneshot().start(100u64.micros());

        Shared {
            i2c,
            mail_buf_0: 0,
            mail_buf_1: 0,
            mail_buf_2: 0,
            mail_buf_3: 0,
        }
    }

    #[task(binds = MachineTimer, priority = 0xff)]
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

                    send_letter(SIM_PRESCALER, 10);
                    send_letter(SIM_LOAD, LF);
                    wait_outbox_empty();

                    send_letter(TASK_MBX_DLN, 0x400);
                    wait_outbox_empty();

                    send_letter(TASK_REP_0_DLN, 0x400);
                    send_letter(TASK_REP_1_DLN, 0x400);
                    send_letter(TASK_REP_2_DLN, 0x400);
                    send_letter(TASK_REP_3_DLN, 0x400);
                    wait_outbox_empty();

                    send_letter(TASK_UPD_0_DLN, 0x400);
                    send_letter(TASK_UPD_1_DLN, 0x400);
                    send_letter(TASK_UPD_2_DLN, 0x400);
                    send_letter(TASK_UPD_3_DLN, 0x400);
                    wait_outbox_empty();

                    send_letter(TASK_CTRL_0_DLN, 0x400);
                    send_letter(TASK_CTRL_1_DLN, 0x400);
                    send_letter(TASK_CTRL_2_DLN, 0x400);
                    send_letter(TASK_CTRL_3_DLN, 0x400);
                    send_letter(SIM_START, 0x0);

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
                    send_letter(SIM_STOP, 0x1);

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

    // Hardware tasks
    #[task(binds = Timer0Cmp, priority = 0x88, shared = [i2c])]
    struct Ctrl0 {
        integral: i32,
        error: i16,
    }
    impl RticTask for Ctrl0 {
        fn init() -> Self {
            Self {
                integral: 0,
                error: 0,
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control0]");
            pend_task(TASK_REP_0_ACK);
            Report0::spawn(()).unwrap();
            ack_task(TASK_CTRL_0_ACK);
        }
    }

    #[task(binds = Timer1Cmp, priority = 0x88, shared = [i2c])]
    struct Ctrl1 {
        integral: i32,
        error: i16,
    }
    impl RticTask for Ctrl1 {
        fn init() -> Self {
            Self {
                integral: 0,
                error: 0,
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control1]");
            pend_task(TASK_REP_1_ACK);
            Report1::spawn(()).unwrap();
            ack_task(TASK_CTRL_1_ACK);
        }
    }

    #[task(binds = Timer2Cmp, priority = 0x88, shared = [i2c])]
    struct Ctrl2 {
        integral: i32,
        error: i16,
    }
    impl RticTask for Ctrl2 {
        fn init() -> Self {
            Self {
                integral: 0,
                error: 0,
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control2]");
            pend_task(TASK_REP_2_ACK);
            Report2::spawn(()).unwrap();
            ack_task(TASK_CTRL_2_ACK);
        }
    }

    #[task(binds = Timer3Cmp, priority = 0x88, shared = [i2c])]
    struct Ctrl3 {
        integral: i32,
        error: i16,
    }
    impl RticTask for Ctrl3 {
        fn init() -> Self {
            Self {
                integral: 0,
                error: 0,
            }
        }
        fn exec(&mut self) {
            sprintln!("[Control3]");
            pend_task(TASK_REP_3_ACK);
            Report3::spawn(()).unwrap();
            ack_task(TASK_CTRL_3_ACK);
        }
    }

    #[task(binds = Mbx, priority = 0xf1, shared = [mail_buf_0, mail_buf_1, mail_buf_2, mail_buf_3])]
    struct Mail {}
    impl RticTask for Mail {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            // IRQ clear
            mmio::write_u32(MAILBOX.ctrl as usize, 0x0002_0000);
            // TODO: more & better abstraction
            for _ in 0..4 {
                // Read inbox
                let addr = mmio::read_u32(MAILBOX.ib.addr as usize);
                let data = mmio::read_u32(MAILBOX.ib.data as usize);

                unsafe {
                    match addr {
                        0x100 => self.shared().mail_buf_0.lock(|mail| *mail = data),
                        0x101 => self.shared().mail_buf_1.lock(|mail| *mail = data),
                        0x102 => self.shared().mail_buf_2.lock(|mail| *mail = data),
                        0x103 => self.shared().mail_buf_3.lock(|mail| *mail = data),
                        _ => sprintln!("Weird letter"),
                    }
                }
                // pop letter?
                mmio::write_u32(MAILBOX.ctrl as usize, 0x0100_0000);
            }
            sprintln!("[Mailbox]");
            pend_task(TASK_UPD_0_ACK);
            Update0::spawn(()).unwrap();
            pend_task(TASK_UPD_1_ACK);
            Update1::spawn(()).unwrap();
            pend_task(TASK_UPD_2_ACK);
            Update2::spawn(()).unwrap();
            pend_task(TASK_UPD_3_ACK);
            Update3::spawn(()).unwrap();
            ack_task(TASK_MBX_ACK);
        }
    }

    // Software tasks
    #[sw_task(priority = 0x11, shared = [mail_buf_0])]
    struct Update0;
    impl RticSwTask for Update0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 0]");

            unsafe {
                let mut nperiod_us: u32 = 0;
                self.shared().mail_buf_0.lock(|m| nperiod_us = *m);
                restart_timer_with_period(MotorIdx::M0, nperiod_us.micros());
                send_letter(TASK_CTRL_0_PER, nperiod_us);
                send_letter(TASK_CTRL_0_DLN, nperiod_us);
            }
            // invalidate these if currently active
            ack_task(TASK_CTRL_0_ACK);
            ack_task(TASK_REP_0_ACK);

            ack_task(TASK_UPD_0_ACK);
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_1])]
    struct Update1;
    impl RticSwTask for Update1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 1]");
            unsafe {
                let mut nperiod_us: u32 = 0;
                self.shared().mail_buf_1.lock(|m| nperiod_us = *m);
                restart_timer_with_period(MotorIdx::M1, nperiod_us.micros());
                send_letter(TASK_CTRL_1_PER, nperiod_us);
                send_letter(TASK_CTRL_1_DLN, nperiod_us);
            }
            // invalidate these if currently active
            ack_task(TASK_CTRL_1_ACK);
            ack_task(TASK_REP_1_ACK);

            ack_task(TASK_UPD_1_ACK);
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_2])]
    struct Update2;
    impl RticSwTask for Update2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 2]");
            unsafe {
                let mut nperiod_us: u32 = 0;
                self.shared().mail_buf_2.lock(|m| nperiod_us = *m);
                restart_timer_with_period(MotorIdx::M2, nperiod_us.micros());
                send_letter(TASK_CTRL_2_PER, nperiod_us);
                send_letter(TASK_CTRL_2_DLN, nperiod_us);
            }
            // invalidate these if currently active
            ack_task(TASK_CTRL_2_ACK);
            ack_task(TASK_REP_2_ACK);

            ack_task(TASK_UPD_2_ACK);
        }
    }

    #[sw_task(priority = 0x11, shared = [mail_buf_3])]
    struct Update3;
    impl RticSwTask for Update3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Update 3]");
            unsafe {
                let mut nperiod_us: u32 = 0;
                self.shared().mail_buf_3.lock(|m| nperiod_us = *m);
                restart_timer_with_period(MotorIdx::M3, nperiod_us.micros());
                send_letter(TASK_CTRL_3_PER, nperiod_us);
                send_letter(TASK_CTRL_3_DLN, nperiod_us);
            }
            // invalidate these if currently active
            ack_task(TASK_CTRL_3_ACK);
            ack_task(TASK_REP_3_ACK);

            ack_task(TASK_UPD_3_ACK);
        }
    }

    #[sw_task(priority = 0x10, shared = [])]
    struct Report0;
    impl RticSwTask for Report0 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 0]");
            ack_task(TASK_REP_0_ACK);
        }
    }

    #[sw_task(priority = 0x10, shared = [])]
    struct Report1;
    impl RticSwTask for Report1 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 1]");
            ack_task(TASK_REP_1_ACK);
        }
    }
    #[sw_task(priority = 0x10, shared = [])]
    struct Report2;
    impl RticSwTask for Report2 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 2]");
            ack_task(TASK_REP_2_ACK);
        }
    }
    #[sw_task(priority = 0x10, shared = [])]
    struct Report3;
    impl RticSwTask for Report3 {
        type SpawnInput = ();
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self, _p: ()) {
            sprintln!("[Report 3]");
            ack_task(TASK_REP_3_ACK);
        }
    }
}
/*
/// Simulator environment address layout
struct AddrSim {
    /// Start the simulation
    start: u32,
    /// Stop the simulation
    stop: u32,
    /// Enables alignment of simulation clock with CPU clock frequency
    prescaler: u32,
    loadfactor: u32,
    seed: u32,
}

/// Task address layout
struct AddrTask {
    period: u32,
    deadline: u32,
    /// * Write 0 to acknowledge
    /// * Write 1 to pend
    ack_pend: u32,
}

/// Task set address layout
struct AddrTaskSet {
    mbx: AddrTask,
    update: [AddrTask; 4],
    control: [AddrTask; 4],
    report: [AddrTask; 4],
}


const fn get_task_addr(idx: u32) -> AddrTask {
    let base = 0x0200_0000 + idx * 0x1_0000;
    let per_offs = 0x0;
    let dl_offs = 0x1;
    let ack_offs = 0x2;

    AddrTask {
        period: base + per_offs,
        deadline: base + dl_offs,
        ack_pend: base + ack_offs,
    }
}

const TASKS: AddrTaskSet = AddrTaskSet {
    mbx: get_task_addr(0),
    update: [
        get_task_addr(1),
        get_task_addr(2),
        get_task_addr(3),
        get_task_addr(4),
    ],
    control: [
        get_task_addr(5),
        get_task_addr(6),
        get_task_addr(7),
        get_task_addr(8),
    ],
    report: [
        get_task_addr(9),
        get_task_addr(10),
        get_task_addr(11),
        get_task_addr(12),
    ],
};

/// Motor addresses
const I2C_M0_ADDR: u8 = 0x10;
const I2C_M1_ADDR: u8 = 0x11;
const I2C_M2_ADDR: u8 = 0x12;
const I2C_M3_ADDR: u8 = 0x13;


const MBX_PRINT_ADDR: u32 = 0x0300_0000;


/// Prescaler
const PS: u32 = 10;
/// Random seed
const SEED: u32 = 0xB0110c55;

const DL_MBX: u32 = 1000;
const DL_CTRL: u32 = 1000;
/// Update deadline
const DL_UPD: u32 = 2000;
/// Report deadline
const DL_REP: u32 = 2000;

/// Simulator hyperparameters
struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

/// Mailbox period
///
/// Inferred from from Mailbox deadline (`DL_MBX`) and load factor (`LF`)
const MBX_PER: u32 = 5 * DL_MBX + (4 * (100 - LF));

// NOTE: 1 x i2c 8-bit read-write (1 addr+1 read+1 addr+1 write)
// blocks for around 140 us (measured from wave)

const PRIO_CTRL: u8 = 4;
const PRIO_MAIL: u8 = 5;
const PRIO_UPD: u8 = 3;
const PRIO_REP: u8 = 1;

static mut MAIL_BUF: [u32; 4] = [0; 4];
static mut CTRL_BUF: [u8; 4] = [0; 4];

// PID state per motor
static mut INTEGRAL: [i32; 4] = [0; 4];
static mut PREV_ERR: [i16; 4] = [0; 4];
#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("\n\r### Starting rt_prof (bare-metal) benchmark ###\n\r");
    let _i2c = I2c::init(4);

    assert!(MBX_PER > DL_MBX, "Mailbox task period too short!\n\r");
    assert!(
        CTRL_TASK_PER_US > DL_CTRL,
        "Control task period too short!\n\r"
    );

    init_intc();

    setup_irq(Interrupt::MachineTimer, u8::MAX);
    setup_irq(Interrupt::MachineExternal, u8::MAX);

    setup_irq(Interrupt::Mbx, PRIO_MAIL);

    setup_irq(Interrupt::Timer0Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer1Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer2Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer3Ovf, PRIO_UPD);

    setup_irq(Interrupt::Timer0Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer1Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer2Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer3Cmp, PRIO_CTRL);

    setup_irq(Interrupt::Ext0, PRIO_REP);
    setup_irq(Interrupt::Ext1, PRIO_REP);
    setup_irq(Interrupt::Ext2, PRIO_REP);
    setup_irq(Interrupt::Ext3, PRIO_REP);

    let timers = &mut [
        Timer::init::<TIMER0_ADDR>().into_periodic(),
        Timer::init::<TIMER1_ADDR>().into_periodic(),
        Timer::init::<TIMER2_ADDR>().into_periodic(),
        Timer::init::<TIMER3_ADDR>().into_periodic(),
    ];

    timers
        .iter_mut()
        .for_each(|t| t.set_period(CTRL_TASK_PER_US.micros()));

    TASKS
        .control
        .iter()
        .for_each(|c| send_letter(c.period, CTRL_TASK_PER_US));
    wait_outbox_empty();

    let mut mtimer = MTimer::instance().into_oneshot();

    sprintln!("Configuration and parameters:");
    sprintln!(" - Runtime (ms)          : {}", SIM_PARAMS.hyperperiod_ms);
    sprintln!(" - Microsecond prescaler : {}", PS);
    sprintln!(" - Randomization seed    : 0x{:X}", SEED);
    sprintln!(" - Load factor  [0-100]  : {}", LF);
    sprintln!(" - Update task DL   (us) : {}", DL_UPD);
    sprintln!(" - Report task DL   (us) : {}", DL_REP);
    sprintln!(" - Control task DL  (us) : {}", DL_CTRL);
    sprintln!(" - Control period   (us) : {}", CTRL_TASK_PER_US);
    sprintln!(" - Mailbox task DL  (us) : {}", DL_MBX);
    sprintln!(" - Mailbox period   (us) : {}", MBX_PER);
    sprintln!("");

    send_letter(TASKS.mbx.deadline, DL_MBX);
    TASKS
        .update
        .iter()
        .for_each(|upd| send_letter(upd.deadline, DL_UPD));
    wait_outbox_empty();
    TASKS
        .control
        .iter()
        .for_each(|ctl| send_letter(ctl.deadline, DL_CTRL));
    wait_outbox_empty();
    TASKS
        .report
        .iter()
        .for_each(|rpt| send_letter(rpt.deadline, DL_REP));
    wait_outbox_empty();

    send_letter(SIM.prescaler, PS);
    send_letter(SIM.loadfactor, LF);
    wait_outbox_empty();

    send_letter(SIM.start, 0x0);

    timers.iter_mut().for_each(Periodic::start);
    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    unsafe { riscv::interrupt::enable() };

    loop {
        wfi();
    }
}

#[inline]
fn wait_outbox_empty() {
    while (mmio::read_u32(MAILBOX.stat as usize) & (0b1 << 2)) == 0 {}
}

#[inline]
fn send_letter(addr: u32, data: u32) {
    mmio::write_u32(MAILBOX.ob.addr as usize, addr);
    mmio::write_u32(MAILBOX.ob.data as usize, data);
    // send letter
    mmio::write_u32(MAILBOX.ctrl as usize, 0x1);
}

#[inline]
fn pend_task(addr: u32) {
    send_letter(addr, 0x1);
}

#[inline]
fn ack_task(addr: u32) {
    send_letter(addr, 0x0);
}

#[core_interrupt(bsp::interrupt::Interrupt::MachineTimer)]
#[allow(non_snake_case)]
fn finish_sim() {
    riscv::interrupt::disable();

    let duration_real_cc = MTimer::instance().into_oneshot().duration().ticks();

    // Terminate scoreboard
    send_letter(SIM.stop, 0x1);

    // Delay to let outbox clear
    for _ in 0..100 {
        nop();
    }

    let instret = riscv::register::minstret::read64();
    let active_time_cc = riscv::register::mcycle::read64();

    sprintln!("\n\rInstructions retired: {instret}, cycles: {active_time_cc}");

    sprintln!(
        "Total time (cc): {}, active time (cc): {},",
        duration_real_cc,
        active_time_cc
    );
    sprintln!(
        "CPU utilization: {}%",
        (active_time_cc * 100) / duration_real_cc
    );

    #[cfg(feature = "rtl-tb")]
    bsp::tb::rtl_tb_signal_ok();
}

fn compute_pid(input: u8, idx: usize) -> u8 {
    // Discrete-time PID controller (per-call integral/derivative coefficients)
    const SETPOINT: i16 = 127;
    const KP: i32 = 1; // proportional gain
    const KI: i32 = 1; // integral gain (per step)
    const KD: i32 = 1; // derivative gain (per step)
    const INTEGRAL_MAX: i32 = 10_000;

    let err: i16 = SETPOINT - input as i16;

    unsafe {
        // Accumulate integral (anti-windup via clamping)
        INTEGRAL[idx] = (INTEGRAL[idx] + err as i32).clamp(-INTEGRAL_MAX, INTEGRAL_MAX);

        let deriv: i32 = (err - PREV_ERR[idx]) as i32;

        let p_term: i32 = KP * (err as i32);
        let i_term: i32 = KI * INTEGRAL[idx];
        let d_term: i32 = KD * deriv;

        PREV_ERR[idx] = err;

        //let mut out: i32 = SETPOINT as i32 + (p_term + i_term + d_term);
        let mut out: i32 = SETPOINT as i32 + (p_term + i_term + d_term);
        if out < 0 {
            out = 0;
        }
        if out > 255 {
            out = 255;
        }

        // used by report tasks
        CTRL_BUF[idx] = input as u8;

        out as u8
    }
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer0Cmp)]
#[allow(non_snake_case)]
fn control_0() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    //unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.read(I2C_M0_ADDR, &mut rbuf); // Read motor status
    });

    let measured_v = u8::from_le_bytes(rbuf);
    let out_v: u8 = compute_pid(measured_v, 0);
    //let out_v: u8 = compute_pid(137, 0);

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.write(I2C_M0_ADDR, &[out_v]); // Write to motor
    });

    // acknowledge this task
    ack_task(TASKS.control[0].ack_pend);
    // Pend report task (HW)
    pend_irq(Interrupt::Ext0);
    // Pend report task (TB)
    pend_task(TASKS.report[0].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Cmp)]
#[allow(non_snake_case)]
fn control_1() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    //unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.read(I2C_M1_ADDR, &mut rbuf);
    });

    let _rdata = u8::from_le_bytes(rbuf);
    let out_v: u8 = compute_pid(_rdata, 1);

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.write(I2C_M1_ADDR, &[out_v]);
    });

    ack_task(TASKS.control[1].ack_pend);
    // Pend report
    pend_irq(Interrupt::Ext1);
    pend_task(TASKS.report[1].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Cmp)]
#[allow(non_snake_case)]
fn control_2() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    //unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];
    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.read(I2C_M2_ADDR, &mut rbuf);
    });

    let _rdata = u8::from_le_bytes(rbuf);

    let out_v: u8 = compute_pid(_rdata, 2);

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.write(I2C_M2_ADDR, &[out_v]);
    });

    ack_task(TASKS.control[2].ack_pend);
    // Pend report
    pend_irq(Interrupt::Ext2);
    pend_task(TASKS.report[2].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Cmp)]
#[allow(non_snake_case)]
fn control_3() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    //unsafe { riscv::interrupt::enable() };

    let mut rbuf: [u8; 1] = [0];

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.read(I2C_M3_ADDR, &mut rbuf);
    });

    let _rdata = u8::from_le_bytes(rbuf);

    let out_v: u8 = compute_pid(_rdata, 3);

    riscv::interrupt::free(|| {
        unsafe { I2c::instance() }.write(I2C_M3_ADDR, &[out_v]);
    });

    ack_task(TASKS.control[3].ack_pend);
    // Pend report
    pend_irq(Interrupt::Ext3);
    pend_task(TASKS.report[3].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}


#[core_interrupt(bsp::interrupt::Interrupt::Timer0Ovf)]
#[allow(non_snake_case)]
fn update_0() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    // invalidate these if currently active
    ack_task(TASKS.control[0].ack_pend);
    ack_task(TASKS.report[0].ack_pend);

    unsafe {
        let nperiod_us = MAIL_BUF[0];
        restart_timer_with_period(MotorIdx::M0, nperiod_us.micros());
        send_letter(TASKS.control[0].period, nperiod_us);
        send_letter(TASKS.control[0].deadline, nperiod_us);
    }

    ack_task(TASKS.update[0].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Ovf)]
#[allow(non_snake_case)]
fn update_1() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    // invalidate these if currently active
    ack_task(TASKS.control[1].ack_pend);
    ack_task(TASKS.report[1].ack_pend);

    unsafe {
        let nperiod_us = MAIL_BUF[1];
        restart_timer_with_period(MotorIdx::M1, nperiod_us.micros());
        send_letter(TASKS.control[1].period, nperiod_us);
        send_letter(TASKS.control[1].deadline, nperiod_us);
    }

    ack_task(TASKS.update[1].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Ovf)]
#[allow(non_snake_case)]
fn update_2() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    // invalidate these if currently active
    ack_task(TASKS.control[2].ack_pend);
    ack_task(TASKS.report[2].ack_pend);

    unsafe {
        let nperiod_us = MAIL_BUF[2];
        restart_timer_with_period(MotorIdx::M2, nperiod_us.micros());
        send_letter(TASKS.control[2].period, nperiod_us);
        send_letter(TASKS.control[2].deadline, nperiod_us);
    }

    ack_task(TASKS.update[2].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Ovf)]
#[allow(non_snake_case)]
fn update_3() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    // invalidate these if currently active
    ack_task(TASKS.control[3].ack_pend);
    ack_task(TASKS.report[3].ack_pend);

    unsafe {
        let nperiod_us = MAIL_BUF[3];
        restart_timer_with_period(MotorIdx::M3, nperiod_us.micros());
        send_letter(TASKS.control[3].period, nperiod_us);
        send_letter(TASKS.control[3].deadline, nperiod_us);
    }

    ack_task(TASKS.update[3].ack_pend);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext0)]
#[allow(non_snake_case)]
fn report_0() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    unsafe {
        let time_now = MTimer::instance().into_oneshot().duration().to_micros();
        let rep_letter = ((time_now as u32) << 16) | ((0u8 as u32) << 8) | (CTRL_BUF[0] as u32);
        send_letter(MBX_PRINT_ADDR, rep_letter);
    }

    ack_task(TASKS.report[0].ack_pend);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext1)]
#[allow(non_snake_case)]
fn report_1() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    unsafe {
        let time_now = MTimer::instance().into_oneshot().duration().to_micros();
        let rep_letter = ((time_now as u32) << 16) | ((1u8 as u32) << 8) | (CTRL_BUF[1] as u32);
        send_letter(MBX_PRINT_ADDR, rep_letter);
    }

    ack_task(TASKS.report[1].ack_pend);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext2)]
#[allow(non_snake_case)]
fn report_2() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    unsafe {
        let time_now = MTimer::instance().into_oneshot().duration().to_micros();
        let rep_letter = ((time_now as u32) << 16) | ((2u8 as u32) << 8) | (CTRL_BUF[2] as u32);
        send_letter(MBX_PRINT_ADDR, rep_letter);
    }

    ack_task(TASKS.report[2].ack_pend);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext3)]
#[allow(non_snake_case)]
fn report_3() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    unsafe {
        let time_now = MTimer::instance().into_oneshot().duration().to_micros();
        let rep_letter = ((time_now as u32) << 16) | ((3u8 as u32) << 8) | (CTRL_BUF[3] as u32);
        send_letter(MBX_PRINT_ADDR, rep_letter);
    }

    ack_task(TASKS.report[3].ack_pend);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Mbx)]
#[allow(non_snake_case)]
fn Mbx() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_MAIL as usize).into());
    unsafe { riscv::interrupt::enable() };

    for _ in 0..4 {
        // Read inbox
        let addr = mmio::read_u32(MAILBOX.ib.addr as usize);
        let data = mmio::read_u32(MAILBOX.ib.data as usize);

        unsafe {
            match addr {
                0x100 => MAIL_BUF[0] = data,
                0x101 => MAIL_BUF[1] = data,
                0x102 => MAIL_BUF[2] = data,
                0x103 => MAIL_BUF[3] = data,
                _ => sprintln!("Weird letter"),
            }
        }

        mmio::write_u32(MAILBOX.ctrl as usize, 0x0100_0000);
    }

    // IRQ clear
    mmio::write_u32(MAILBOX.ctrl as usize, 0x0002_0000);

    pend_task(TASKS.update[0].ack_pend);
    // Pend update
    pend_irq(Interrupt::Timer0Ovf);

    pend_task(TASKS.update[1].ack_pend);
    pend_irq(Interrupt::Timer1Ovf);

    pend_task(TASKS.update[2].ack_pend);
    pend_irq(Interrupt::Timer2Ovf);

    pend_task(TASKS.update[3].ack_pend);
    pend_irq(Interrupt::Timer3Ovf);

    // Ack MBX task
    ack_task(TASKS.mbx.ack_pend);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[unsafe(export_name = "DefaultHandler")]
fn default_handler() {
    sprintln!("Hit default handler (unmapped interrupt)!");
    bsp::tb::rtl_tb_signal_fail();
}

pub fn init_intc() {
    // HACK: clear mintstatus, required for zeroHETI
    unsafe { bsp::register::mintstatus::write(0.into()) };

    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::Clic;
        // Set level bits to 8
        Clic::smclicconfig().set_mnlbits(8);
    }
}

/// Setup `irq` for use with some basic defaults
///
/// Copy and customize this function if you need more involved configurations.
pub fn setup_irq(irq: impl InterruptNumber, _level: u8) {
    log::debug!("Set up IRQ (id = {})", irq.number());
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::{Clic, Polarity, Trig};

        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(_level);
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::{Edfic, Pol, Trig};

        Edfic::line(irq.number()).set_pol(Pol::Pos);
        Edfic::line(irq.number()).set_trig(Trig::Edge);
        Edfic::line(irq.number()).enable();
        Edfic::line(irq.number()).set_dl(0xffff_ffff);
    }
}

pub fn pend_irq(irq: impl InterruptNumber) {
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::CLIC;
        unsafe { CLIC::ip(irq).pend() }
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::Edfic;
        Edfic::line(irq.number()).pend();
    }
}
*/
