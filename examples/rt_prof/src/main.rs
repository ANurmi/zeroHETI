#![no_main]
#![no_std]
#![allow(static_mut_refs)]

mod mailbox;

use bsp::rt as _;
#[rtic::app(device = bsp)]

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

    // TODO: move this to Mbx HAL
    #[inline]
    fn send_letter(addr: u32, data: u32) {
        mmio::write_u32(MAILBOX.ob.addr as usize, addr);
        mmio::write_u32(MAILBOX.ob.data as usize, data);
        // send letter
        mmio::write_u32(MAILBOX.ctrl as usize, 0x1);
    }

    /// Runtime in millis, read from env RUNTIME_MS
    const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;
    /// Load factor, read from env LOAD_FACTOR
    const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
    /// Control task period
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
    }

    #[init]
    fn init() -> Shared {
        let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
        sprintln!("\n\r### Starting rt_prof (rtic) benchmark ###\n\r");
        let i2c = I2c::init(4);

        MTimer::instance().into_oneshot().start(100u64.micros());

        Shared { i2c }
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

                    unsafe {
                        // Clear instruction & cycle counters
                        riscv::register::minstret::write(0);
                        riscv::register::mcycle::write(0);
                        riscv::register::minstreth::write(0);
                        riscv::register::mcycleh::write(0);
                    }
                }
                Some(start_time) => {
                    sprintln!("MachineTimer::Teardown");

                    let duration_real_cc = MTimer::instance().into_oneshot().duration().ticks();

                    // Terminate scoreboard
                    //send_letter(SIM.stop, 0x1);

                    let instret = riscv::register::minstret::read64();
                    let active_time_cc = riscv::register::mcycle::read64();

                    // Delay to let outbox clear
                    for _ in 0..100 {
                        nop();
                    }

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
            sprintln!("Ding0");
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
            sprintln!("Ding1");
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
            sprintln!("Ding2");
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
            sprintln!("Ding3");
        }
    }

    #[task(binds = Mbx, priority = 0xf1, shared = [])]
    struct Mail {}
    impl RticTask for Mail {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            sprintln!("Mbx");
            //TODO: how to spawn sw tasks
            //Update0::spawn(()).unwrap();
        }
    }

    #[task(binds = Timer0Ovf, priority = 0x11, shared = [])]
    struct Update0 {}
    impl RticTask for Update0 {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) {
            sprintln!("Dong0");
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

const SIM: AddrSim = AddrSim {
    start: 0x0100_0000,
    stop: 0x0100_0001,
    prescaler: 0x0100_0002,
    loadfactor: 0x0100_0003,
    seed: 0x0100_0004,
};

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

fn restart_timer_with_period(idx: MotorIdx, nperiod: timer_group::Duration) {
    let timer_addr = TIMER0_ADDR + (idx as usize) * TIMER_SEP;
    let mut timer = unsafe { Timer::instance_dyn(timer_addr) }.into_periodic();
    timer.cancel();
    timer.set_period(nperiod);
    timer.start();
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
