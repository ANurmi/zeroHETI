#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fclic-hetic, -Fclic-clic, -Fclic-edfic"
);

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    i2c::I2c,
    interrupt::{CoreInterrupt, ExternalInterrupt},
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mtimer::*,
    nested_interrupt,
    riscv::{self, asm::wfi},
    rt::entry,
    sprintln,
    tb::signal_pass,
    timer_group::{Periodic, Timer},
};
use core::i16;
use fugit::{ExtU32, ExtU64};
use motor_control::{
    mailbox::{Mailbox, Motor::*},
    *,
};

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

const MBX_DL_US: u32 = 3200;
const WRN_DL_US: u32 = 1500;
const REP_DL_US: u32 = 1200;

// Global variables
static mut I2C: Option<I2c> = None;
static mut MBX: Mailbox = unsafe { Mailbox::instance() };

static mut SPEED_REAL: [i32; 4] = [0, 0, 0, 0];
static mut VOLTAGE_TARGET: [i32; 4] = [0, 0, 0, 0];

static mut INTEGRAL: [i32; 4] = [0, 0, 0, 0];
static mut PREV_ERR: [i32; 4] = [0, 0, 0, 0];

static mut START_TIME: Option<u64> = None;
static mut END_TIME: Option<u64> = None;

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    let riscv_isa = core::env!("RISCV_ISA");
    sprintln!("[Motor control demo] ISA = {riscv_isa}");

    // Init I2C into the global context
    unsafe { I2C.replace(I2c::init(4)) };

    // HACK: clear mintstatus
    unsafe { bsp::register::mintstatus::write(0.into()) };

    // Set level bits to 8
    #[cfg(feature = "intc-clic")]
    bsp::clic::Clic::smclicconfig().set_mnlbits(8);

    // Setup HetIc / CLIC with priority
    #[cfg(any(feature = "intc-hetic", feature = "intc-clic"))]
    {
        setup_irq(CoreInterrupt::MachineTimer, 0xFF);
        setup_irq(ExternalInterrupt::Timer0Cmp, 0x88);
        setup_irq(ExternalInterrupt::Timer1Cmp, 0x88);
        setup_irq(ExternalInterrupt::Timer2Cmp, 0x88);
        setup_irq(ExternalInterrupt::Timer3Cmp, 0x88);
        setup_irq(ExternalInterrupt::Mbx, 0x3);
        setup_irq(ExternalInterrupt::Ext0, 0x10);
        setup_irq(ExternalInterrupt::Ext1, 0x10);
        setup_irq(ExternalInterrupt::Ext2, 0x10);
        setup_irq(ExternalInterrupt::Ext3, 0x10);
    }
    // Setup EDFIC with deadlines
    #[cfg(feature = "intc-edfic")]
    {
        sprintln!(
            "Setup IRQ deadlines (us): MBX = {}, WRN = {}, REP = {}",
            MBX_DL_US,
            WRN_DL_US,
            REP_DL_US
        );
        // MachineTimer should have the shortest deadline to ensure it can preempt other
        // interrupts and print the log before simulation termination
        setup_irq_dl(CoreInterrupt::MachineTimer, 0x0);
        setup_irq_dl(ExternalInterrupt::Timer0Cmp, REP_DL_US);
        setup_irq_dl(ExternalInterrupt::Timer1Cmp, REP_DL_US);
        setup_irq_dl(ExternalInterrupt::Timer2Cmp, REP_DL_US);
        setup_irq_dl(ExternalInterrupt::Timer3Cmp, REP_DL_US);
        setup_irq_dl(ExternalInterrupt::Mbx, MBX_DL_US);
        setup_irq_dl(ExternalInterrupt::Ext0, WRN_DL_US);
        setup_irq_dl(ExternalInterrupt::Ext1, WRN_DL_US);
        setup_irq_dl(ExternalInterrupt::Ext2, WRN_DL_US);
        setup_irq_dl(ExternalInterrupt::Ext3, WRN_DL_US);
    }

    let mut mtimer = MTimer::instance().into_oneshot();

    unsafe { START_TIME.replace(mtimer.counter()) };

    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    let timers = &mut [
        Timer::init::<TIMER0_ADDR>().into_periodic(),
        Timer::init::<TIMER1_ADDR>().into_periodic(),
        Timer::init::<TIMER2_ADDR>().into_periodic(),
        Timer::init::<TIMER3_ADDR>().into_periodic(),
    ];

    timers[0].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());
    timers[1].set_period_offset(
        REP_TASK_PER_US.micros(),
        REP_TASK_OFS_US.micros() - (1 * 250u32).micros(),
    );
    timers[2].set_period_offset(
        REP_TASK_PER_US.micros(),
        REP_TASK_OFS_US.micros() - (2 * 250u32).micros(),
    );
    timers[3].set_period_offset(
        REP_TASK_PER_US.micros(),
        REP_TASK_OFS_US.micros() - (3 * 250u32).micros(),
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
        I2C.as_mut().unwrap().write(0x0, &[1]);
        riscv::interrupt::enable();
    }

    loop {
        wfi();
    }
}

#[nested_interrupt]
unsafe fn Timer0Cmp() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::disable();
        I2C.as_mut().unwrap().read(M0_ADDR.stat, &mut rbuf);
        riscv::interrupt::enable();
    }
    let m0_speed: i32 = i32::from_le_bytes(rbuf);

    let time = MTimer::instance().counter();

    unsafe {
        SPEED_REAL[0] = m0_speed;
        MBX.write_time_and_stat(time, m0_speed as u32, M0);
    }
}

#[nested_interrupt]
unsafe fn Timer1Cmp() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| {
            I2C.as_mut().map(|i2c| i2c.read(M1_ADDR.stat, &mut rbuf));
        });
    }
    let m1_speed: i32 = i32::from_le_bytes(rbuf);

    let time = MTimer::instance().counter();

    unsafe {
        SPEED_REAL[1] = m1_speed;
        MBX.write_time_and_stat(time, m1_speed as u32, M1);
    }
}

#[nested_interrupt]
unsafe fn Timer2Cmp() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M2_ADDR.stat, &mut rbuf)));
    }
    let m2_speed: i32 = i32::from_le_bytes(rbuf);

    let time = MTimer::instance().counter();

    unsafe {
        SPEED_REAL[2] = m2_speed;
        MBX.write_time_and_stat(time, m2_speed as u32, M2);
    }
}

#[nested_interrupt]
unsafe fn Timer3Cmp() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M3_ADDR.stat, &mut rbuf)));
    }
    let m3_speed: i32 = i32::from_le_bytes(rbuf);

    let time = MTimer::instance().counter();

    unsafe {
        SPEED_REAL[3] = m3_speed;
        MBX.write_time_and_stat(time, m3_speed as u32, M3);
    }
}

#[nested_interrupt]
unsafe fn Mbx() {
    let mail = unsafe { MBX.read_inbox() };
    let bytes: [u8; 4] = mail.to_be_bytes();
    //let bytes: [u8; 4] = [0, 0, 0, mail.to_be_bytes()[0]];

    for i in 0..4 {
        unsafe {
            riscv::interrupt::free(|| {
                I2C.as_mut()
                    .map(|i2c| i2c.write(2 + i * 3, &[0u8, bytes[i as usize]]));
                VOLTAGE_TARGET[i as usize] = ((bytes[i as usize]) as i32) << 8;
            });
        }
    }

    unsafe {
        MBX.ack_irq();
    }
}

#[nested_interrupt]
unsafe fn Ext0() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M0_ADDR.stat, &mut rbuf)));
    }
    let m0_speed_now: i32 = i32::from_le_bytes(rbuf);

    unsafe {
        let bytes: [u8; 2] = compute_control(0, m0_speed_now).to_le_bytes();
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.write(M0_ADDR.tune, &bytes)));
    }
}

#[nested_interrupt]
unsafe fn Ext1() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M1_ADDR.stat, &mut rbuf)));
    }
    let m1_speed_now: i32 = i32::from_le_bytes(rbuf);

    unsafe {
        let bytes: [u8; 2] = compute_control(1, m1_speed_now).to_le_bytes();
        riscv::interrupt::disable();
        I2C.as_mut().unwrap().write(M1_ADDR.tune, &bytes);
        riscv::interrupt::enable();
    }
}

#[nested_interrupt]
unsafe fn Ext2() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M2_ADDR.stat, &mut rbuf)));
    }
    let m2_speed_now: i32 = i32::from_le_bytes(rbuf);

    unsafe {
        let bytes: [u8; 2] = compute_control(2, m2_speed_now).to_le_bytes();
        riscv::interrupt::disable();
        I2C.as_mut().unwrap().write(M2_ADDR.tune, &bytes);
        riscv::interrupt::enable();
    }
}

#[nested_interrupt]
unsafe fn Ext3() {
    let mut rbuf = [0; 4];

    unsafe {
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.read(M3_ADDR.stat, &mut rbuf)));
    }
    let m3_speed_now: i32 = i32::from_le_bytes(rbuf);

    unsafe {
        let bytes: [u8; 2] = compute_control(3, m3_speed_now).to_le_bytes();
        riscv::interrupt::free(|| I2C.as_mut().map(|i2c| i2c.write(M3_ADDR.tune, &bytes)));
    }
}

#[inline]
/// Compute tuning voltage to control motor power.
unsafe fn compute_control(idx: usize, speed_now: i32) -> i16 {
    let res: i32 = 10_000; // mOhm
    let v_target = unsafe { VOLTAGE_TARGET }[idx];
    let p_target: i32 = i32::pow(v_target, 2) / res; // mW

    // Assume Power (mW) and Speed (RPM) directly correlated
    let error = p_target - speed_now;

    let mut v_out: i16 = usqrt4((error / res).abs() as u32) as i16;

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

#[bsp::core_interrupt(CoreInterrupt::MachineTimer)]
unsafe fn MachineTimer() {
    unsafe { END_TIME.replace(MTimer::instance().counter().into()) };

    // Explicitly terminate simulation to print task log
    unsafe { I2C.as_mut() }.map(|i2c| i2c.write(0x0, &[0]));

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
