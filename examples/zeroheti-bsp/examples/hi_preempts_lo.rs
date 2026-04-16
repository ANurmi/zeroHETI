//! Print hello over UART
#![no_main]
#![no_std]
mod common;

use crate::common::{init_intc, pend_irq};
use fugit::ExtU64;
use riscv::InterruptNumber;
use zeroheti_bsp::{
    CPU_FREQ_HZ, apb_uart::ApbUart, interrupt::Interrupt, mtimer::MTimer, rt::entry, sprintln,
};

/// Indicates that the high-level interrupt was called
static mut HI_VISITED: bool = false;
/// Indicates that the low-level interrupt was called
static mut LO_VISITED: bool = false;

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    // Print example meta
    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    init_intc();

    // Setup mtimer for timeout, Ext0 for high-level interrupt and Ext1 for low-level interrupt.
    setup_irq(Interrupt::MachineTimer, 0xff);
    setup_irq(Interrupt::Ext0, 2);
    setup_irq(Interrupt::Ext1, 1);

    // Check results after timeout
    let mut mtimer = MTimer::instance().into_oneshot();
    let timeout = 1000u64.micros();
    mtimer.start(timeout);

    unsafe { riscv::interrupt::enable() };

    // Dispatch both low-level and high-level interrupt in that order.
    // After mtimer timeout, the mtimer handler ensures that both interrupts
    // were acknowledged indicating that high preempted low.
    pend_irq(Interrupt::Ext1);
    pend_irq(Interrupt::Ext0);

    // It is considered a failure, if execution falls through the interrupt
    // handlers (low should block until preempted by high).
    zeroheti_bsp::tb::signal_fail(Some(&mut serial));

    loop {}
}

/// A.k.a., 'high_level'
#[zeroheti_bsp::nested_interrupt]
#[allow(non_snake_case)]
fn Ext0() {
    unsafe { HI_VISITED = true };
    sprintln!("hi enter");
}

/// A.k.a., 'low_level'
#[zeroheti_bsp::core_interrupt(Interrupt::Ext1)]
#[allow(non_snake_case)]
fn low_level() {
    unsafe { LO_VISITED = true };
    sprintln!("lo enter");

    // Enable preemption
    unsafe { core::arch::asm!("csrsi mstatus, 8") };

    // Block, allowing high-level interrupt to preempt this before test timeout
    loop {}
}

#[zeroheti_bsp::core_interrupt(Interrupt::MachineTimer)]
fn timeout() {
    assert!(
        unsafe { HI_VISITED },
        "high-level interrupt should have been visited"
    );
    assert!(
        unsafe { LO_VISITED },
        "low-level interrupt should have been visited"
    );

    // If all assertions are true, signal pass
    zeroheti_bsp::tb::signal_pass(None);
}

pub fn setup_irq(irq: impl InterruptNumber, lvl: u8) {
    #[cfg(feature = "intc-clic")]
    {
        use zeroheti_bsp::clic::{Clic, Polarity, Trig};

        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(lvl);
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-hetic")]
    {
        use zeroheti_bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level_prio(lvl);
        Hetic::line(irq.number()).enable();
    }
    #[cfg(feature = "intc-edfic")]
    {
        use zeroheti_bsp::edfic::{Edfic, Pol, Trig};

        Edfic::line(irq.number()).set_pol(Pol::Pos);
        Edfic::line(irq.number()).set_trig(Trig::Edge);
        Edfic::line(irq.number()).enable();
        Edfic::line(irq.number()).set_dl(((u8::MAX - lvl) as u32) << 24);
    }
}
