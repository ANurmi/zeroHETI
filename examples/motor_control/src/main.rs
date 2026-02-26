#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fintc-hetic, -Fintc-clic, -Fintc-edfic"
);

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    i2c::I2c,
    interrupt::{CoreInterrupt, ExternalInterrupt},
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mtimer::*,
    nested_interrupt,
    register::mintthresh,
    riscv::{self, asm::wfi},
    rt::entry,
    sprintln,
    tb::signal_pass,
    timer_group::{Periodic, Timer},
};
use core::i16;
use fugit::{ExtU32, ExtU64};
use motor_control::{
    I2C_ADDRS,
    mailbox::{Mailbox, Motor::*},
    *,
};

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams { hyperperiod_ms: 20 };

const REP_TASK_PER_US: u32 = 4000;
const REP_TASK_OFS_US: u32 = 3900;

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

static mut START_TIME: Option<u64> = None;
static mut END_TIME: Option<u64> = None;

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    let riscv_isa = core::env!("RISCV_ISA");
    sprintln!("[Motor control demo] ISA = {riscv_isa}");

    let mut i2c = I2c::init(4);

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

    loop {
        wfi();
    }
}

/// Interrupt free on CLIC & Hetic. nop on EDFIC.
fn free<R>(f: impl FnOnce() -> R) -> R {
    match () {
        //#[cfg(not(feature = "intc-edfic"))]
        () => riscv::interrupt::free(|| f()),
        /*#[cfg(feature = "intc-edfic")]
        () => f(),*/
    }
}

#[nested_interrupt]
unsafe fn Timer0Cmp() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Timer0Cmp.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[0].stat, &mut rbuf));

    let m0_speed = u32::from_le_bytes(rbuf);
    let time = MTimer::instance().counter();

    // SAFETY: there are no other users of SPEED_REAL[0]
    unsafe { SPEED_REAL[0] = m0_speed };
    free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m0_speed as u32, M0) });

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Timer1Cmp() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Timer1Cmp.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[1].stat, &mut rbuf));

    let m1_speed = u32::from_le_bytes(rbuf);
    let time = MTimer::instance().counter();

    // SAFETY: there are no other users of SPEED_REAL[1]
    unsafe { SPEED_REAL[1] = m1_speed };
    free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m1_speed as u32, M1) });

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Timer2Cmp() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Timer2Cmp.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
         unsafe { I2c::instance() }.read(I2C_ADDRS.motors[2].stat, &mut rbuf));

    let m2_speed = u32::from_le_bytes(rbuf);
    let time = MTimer::instance().counter();

    // SAFETY: there are no other users of SPEED_REAL[2]
    unsafe { SPEED_REAL[2] = m2_speed };
    free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m2_speed as u32, M2)});

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Timer3Cmp() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Timer3Cmp.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
         unsafe { I2c::instance() }.read(I2C_ADDRS.motors[3].stat, &mut rbuf));

    let m3_speed = u32::from_le_bytes(rbuf);
    let time = MTimer::instance().counter();

    // SAFETY: there are no other users of SPEED_REAL[3]
    unsafe { SPEED_REAL[3] = m3_speed };
    free(||
        // SAFETY: other users of MBX are excluded
        unsafe { MBX.write_time_and_stat(time, m3_speed as u32, M3)});

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Mbx() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Mbx.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    // SAFETY: the inbox is not read by any other context
    let mail = unsafe { MBX.read_inbox() };
    let bytes: [u8; 4] = mail.to_be_bytes();

    for i in 0usize..4 {
        free(|| {
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

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Ext0() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Ext0.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[0].stat, &mut rbuf));

    let m0_speed_now = u32::from_le_bytes(rbuf);
    let bytes: [u8; 2] = compute_control(0, m0_speed_now).to_le_bytes();
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(I2C_ADDRS.motors[0].tune, &bytes));

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Ext1() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Ext1.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[1].stat, &mut rbuf));

    let m1_speed_now = u32::from_le_bytes(rbuf);
    let bytes: [u8; 2] = compute_control(1, m1_speed_now).to_le_bytes();
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(I2C_ADDRS.motors[1].tune, &bytes));

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Ext2() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Ext2.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[2].stat, &mut rbuf));

    let m2_speed_now = u32::from_le_bytes(rbuf);
    let bytes: [u8; 2] = compute_control(2, m2_speed_now).to_le_bytes();
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(I2C_ADDRS.motors[2].tune, &bytes));

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[nested_interrupt]
unsafe fn Ext3() {
    // Save mintthresh
    #[cfg(feature = "intc-edfic")]
    let current = {
        use riscv_rt::InterruptNumber;

        let dl = bsp::edfic::Edfic::line(ExternalInterrupt::Ext3.number()).dl();
        let ceiling = 255 - (dl >> 8);
        mintthresh::write((ceiling as usize).into())
    };

    let mut rbuf = [0; 4];
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.read(I2C_ADDRS.motors[3].stat, &mut rbuf));

    let m3_speed_now = u32::from_le_bytes(rbuf);
    let bytes: [u8; 2] = compute_control(3, m3_speed_now).to_le_bytes();
    free(||
        // SAFETY: other users of I2C are excluded
        unsafe { I2c::instance() }.write(I2C_ADDRS.motors[3].tune, &bytes));

    // Restore mintthresh
    #[cfg(feature = "intc-edfic")]
    mintthresh::write((current as usize).into());
}

#[inline]
/// Compute tuning voltage to control motor power.
fn compute_control(idx: usize, speed_now: u32) -> i16 {
    // Resistance in mOhm
    let res = 10_000;
    let v_target = free(||
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
