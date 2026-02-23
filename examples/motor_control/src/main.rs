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
use fugit::{ExtU32, ExtU64};
use motor_control::{
    mailbox::{Mailbox, Motor::*},
    *,
};

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams { hyperperiod_ms: 1 };

const REP_TASK_PER_US: u32 = 5000;
const REP_TASK_OFS_US: u32 = 4700;

// Global variables
static mut I2C: Option<I2c> = None;
static mut MBX: Mailbox = unsafe { Mailbox::instance() };

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    let riscv_isa = core::env!("RISCV_ISA");
    sprintln!("[Motor control demo] ISA = {riscv_isa}");

    let mut i2c = I2c::init(2);

    // HACK: clear mintstatus
    unsafe { bsp::register::mintstatus::write(0.into()) };

    // Set level bits to 8
    #[cfg(feature = "intc-clic")]
    bsp::clic::Clic::smclicconfig().set_mnlbits(8);

    setup_irq(CoreInterrupt::MachineTimer, 0x88);
    setup_irq(ExternalInterrupt::Timer0Cmp, 0x10);
    setup_irq(ExternalInterrupt::Timer1Cmp, 0x10);
    setup_irq(ExternalInterrupt::Timer2Cmp, 0x10);
    setup_irq(ExternalInterrupt::Timer3Cmp, 0x10);
    setup_irq(ExternalInterrupt::Mbx, 0x1A);

    let mut mtimer = MTimer::instance().into_oneshot();
    //mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    // Start the simulation
    mtimer.start(20u64.millis());

    let timers = &mut [
        Timer::init::<TIMER0_ADDR>().into_periodic(),
        Timer::init::<TIMER1_ADDR>().into_periodic(),
        Timer::init::<TIMER2_ADDR>().into_periodic(),
        Timer::init::<TIMER3_ADDR>().into_periodic(),
    ];

    timers[0].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());
    timers[1].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());
    timers[2].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());
    timers[3].set_period_offset(REP_TASK_PER_US.micros(), REP_TASK_OFS_US.micros());

    unsafe {
        // clear instruction & cycle counters
        riscv::register::minstret::write(0);
        riscv::register::mcycle::write(0);
        riscv::register::minstreth::write(0);
        riscv::register::mcycleh::write(0);
        // Global enable
        riscv::interrupt::enable();
    }

    // Start periodic timers
    timers.iter_mut().for_each(Periodic::start);

    // Start sim
    i2c.write(0x0, &[1]);

    /*
        // Read motor states
        let mut rbuf = [0; 4];
        i2c.read(0x2, &mut rbuf);
        let m0_speed = u32::from_le_bytes(rbuf);

        i2c.read(0x4, &mut rbuf);
        let m1_speed = u32::from_le_bytes(rbuf);

        i2c.read(0x6, &mut rbuf);
        let m2_speed = u32::from_le_bytes(rbuf);

        i2c.read(0x8, &mut rbuf);
        let m3_speed = u32::from_le_bytes(rbuf);

        sprintln!("Motor speeds: M0 = {m0_speed}, M1 = {m1_speed}, M2 = {m2_speed}, M3 = {m3_speed}");
    */
    /*
     * 4 motors.
     *
     * status: read: current motor speed -> u32 (4 byte transaction), [0, 2^32]
     * -> [0, "max speed" = ~35_000], "RPM".
     *
     * control: write: controls motor
     * voltage. Write = 4 byte transaction (32-bit). In byte: [0, 2^32] ->
     * [0, ~20_000] "mV".
     *
     * Ideal: power should be mW == RPM.
     *
     * If overvoltage sent -> sim will complain that about max. voltage
     *
     * Internal address mapping
     * 0: 31'h0, sim_en
     * 1: reserved
     * 2: M0 status
     * 3: M0 control
     * 4: M1 status
     * 5: M1 control
     * 6: M2 status
     * 7: M2 control
     * 8: M3 status
     * 9: M3 control
     */

    unsafe { I2C.replace(i2c) };

    loop {
        wfi();
    }
}

#[nested_interrupt]
unsafe fn Timer0Cmp() {
    unsafe {
        MBX.write_time_and_stat(0u64, 1, M0);
    }
}

#[nested_interrupt]
unsafe fn Timer1Cmp() {
    // Read motor state
    let mut rbuf = [0; 4];


    unsafe {
        //I2C.read(0x2, &mut rbuf);
        I2C.as_mut().unwrap().read(0x2, &mut rbuf);
    }
    let m0_speed = u32::from_le_bytes(rbuf);

    unsafe {
        MBX.write_time_and_stat(0u64, m0_speed, M1);
    }
}

#[nested_interrupt]
unsafe fn Timer2Cmp() {
    unsafe {
        MBX.write_time_and_stat(0u64, 1, M1);
    }
}

#[nested_interrupt]
unsafe fn Timer3Cmp() {
    unsafe {
        MBX.write_time_and_stat(0u64, 1, M3);
    }
}

#[nested_interrupt]
unsafe fn Mbx() {
    unsafe {
        MBX.ack_irq();
    }
}

#[bsp::core_interrupt(CoreInterrupt::MachineTimer)]
unsafe fn MachineTimer() {
    signal_pass(None);
}
