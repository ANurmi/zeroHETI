//! Set 4 timers to trigger consecutively, each setting a flag. In main, check
//! for flags' presence until pass or failure on timeout.
#![no_main]
#![no_std]
mod common;

use core::{file, ptr};

use fugit::{ExtU32, ExtU64};
use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC,
    apb_uart::ApbUart,
    asm_delay,
    interrupt::Interrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mtimer::MTimer,
    rt::entry,
    sprintln,
    timer_group::Timer,
};

use crate::common::{init_intc, setup_irq, tear_irq};

static mut IRQ_RECVD: u64 = 0;

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", file!(), env!("RISCV_EXTS"));

    init_intc();
    setup_irq(Interrupt::Timer0Cmp);
    setup_irq(Interrupt::Timer1Cmp);
    setup_irq(Interrupt::Timer2Cmp);
    setup_irq(Interrupt::Timer3Cmp);
    setup_irq(Interrupt::MachineTimer);

    let mut mtimer = MTimer::instance().into_oneshot();
    let mut timers = [
        Timer::init::<TIMER0_ADDR>().into_periodic(),
        Timer::init::<TIMER1_ADDR>().into_periodic(),
        Timer::init::<TIMER2_ADDR>().into_periodic(),
        Timer::init::<TIMER3_ADDR>().into_periodic(),
    ];

    // Setup & start all timers
    timers
        .iter_mut()
        .enumerate()
        .for_each(|(idx, t)| t.set_period_next(40u32.micros(), (10u32 * idx as u32).micros()));
    timers.iter_mut().for_each(|t| t.start());
    mtimer.start(100u64.micros());

    unsafe { riscv::interrupt::enable() };

    while (unsafe { IRQ_RECVD } & 0b1111) != 0b1111 {}

    zeroheti_bsp::tb::signal_pass(Some(&mut serial));

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer0Cmp)]
fn timer0() {
    // sprintln!("t0");

    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER0_ADDR>() }.disable();
    tear_irq(Interrupt::Timer0Cmp);

    // Record that the particular line was raised
    let mut val = unsafe { ptr::read_volatile(&raw const (IRQ_RECVD) as *const _) };
    val |= 0b1u64 << 0;
    unsafe { ptr::write_volatile(&raw mut (IRQ_RECVD) as *mut _, val) };
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer1Cmp)]
fn timer1() {
    // sprintln!("t1");

    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER1_ADDR>() }.disable();
    tear_irq(Interrupt::Timer1Cmp);

    // Record that the particular line was raised
    let mut val = unsafe { ptr::read_volatile(&raw const (IRQ_RECVD) as *const _) };
    val |= 0b1u64 << 1;
    unsafe { ptr::write_volatile(&raw mut (IRQ_RECVD) as *mut _, val) };
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer2Cmp)]
fn timer2() {
    // sprintln!("t2");

    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER2_ADDR>() }.disable();
    tear_irq(Interrupt::Timer2Cmp);

    // Record that the particular line was raised
    let mut val = unsafe { ptr::read_volatile(&raw const (IRQ_RECVD) as *const _) };
    val |= 0b1u64 << 2;
    unsafe { ptr::write_volatile(&raw mut (IRQ_RECVD) as *mut _, val) };
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer3Cmp)]
fn timer3() {
    // sprintln!("t3");

    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER3_ADDR>() }.disable();
    tear_irq(Interrupt::Timer3Cmp);

    // Record that the particular line was raised
    let mut val = unsafe { ptr::read_volatile(&raw const (IRQ_RECVD) as *const _) };
    val |= 0b1u64 << 3;
    unsafe { ptr::write_volatile(&raw mut (IRQ_RECVD) as *mut _, val) };
}

#[zeroheti_bsp::core_interrupt(Interrupt::MachineTimer)]
fn timeout() {
    sprintln!("timeout");
    let bits = unsafe { ptr::read(&raw const IRQ_RECVD) };
    sprintln!(
        "stat - t0:{} t1:{} t2:{} t3:{}",
        (bits & (0b1 << 0)) >> 0,
        (bits & (0b1 << 1)) >> 1,
        (bits & (0b1 << 2)) >> 2,
        (bits & (0b1 << 3)) >> 3,
    );
    zeroheti_bsp::tb::signal_fail(None);
}
