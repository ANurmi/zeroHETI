#![no_main]
#![no_std]
mod common;

use core::file;

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC,
    apb_uart::ApbUart,
    asm_delay,
    cfg_regs::CfgRegs,
    fugit::{ExtU32, ExtU64},
    interrupt::Interrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER4_ADDR},
    mmio,
    mtimer::{MTimer, OneShot},
    rt::entry,
    sprintln,
    timer_group::Timer,
    timer_queue::TimerQueue,
};

use crate::common::{init_intc, setup_irq, tear_irq};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", file!(), env!("RISCV_EXTS"));
    let cfg_regs = CfgRegs::init();
    let tq = TimerQueue::init();

    init_intc();
    setup_irq(Interrupt::Timer0Cmp);
    setup_irq(Interrupt::Timer4Cmp);
    setup_irq(Interrupt::TqFull);
    setup_irq(Interrupt::TqNFull);
    setup_irq(Interrupt::MachineTimer);

    let mut mtimer = MTimer::instance();
    let mut timer = Timer::init::<TIMER0_ADDR>().into_periodic();

    timer.set_period(20u32.micros());
    timer.start();
    mtimer.start(100u64.micros());

    unsafe { riscv::interrupt::enable() };

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer0Cmp)]
fn timer0() {
    TimerQueue::instance().push_rel(Interrupt::Timer4Cmp, 1u32.micros());
    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER0_ADDR>() }.disable();
    tear_irq(Interrupt::Timer0Cmp);
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer4Cmp)]
fn timer4() {
    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER4_ADDR>() }.disable();
    tear_irq(Interrupt::Timer4Cmp);
    zeroheti_bsp::tb::signal_pass(None);
}

#[zeroheti_bsp::core_interrupt(Interrupt::MachineTimer)]
fn timeout() {
    sprintln!("timeout");
    zeroheti_bsp::tb::signal_fail(None);
}
