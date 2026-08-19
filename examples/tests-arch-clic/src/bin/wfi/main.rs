//! clicwfi-01 — `wfi` behaves like a nop and wakes without trapping when an
//! interrupt is pending and `mstatus.mie = 0`.
#![no_main]
#![no_std]

use bsp::{interrupt::Interrupt, rt as riscv_rt, rt::entry, sprintln};

use tests_arch_clic::*;

/// Number of times the Ext0 handler has run. Single-hart test, accessed only
/// from `main` and the Ext0 handler.
static mut EXT0_FIRES: u32 = 0;

#[entry]
fn main() -> ! {
    let mut failures = 0;

    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 1);
    disable_mie();

    sprintln!("clicwfi-01: wfi wakes without trapping when an interrupt is pending and mie=0");

    // Phase 1: with Ext0 pending and mie=0, `wfi` must wake up (pending
    // interrupt) but NOT trap: the handler must not be called and the pending
    // bit must stay asserted. A hung `wfi` shows up as a simulation timeout.
    pend(Interrupt::Ext0);
    let d = Deadline::start(50);
    bsp::riscv::asm::wfi();
    d.spin_full();
    check(
        &mut failures,
        "handler not called (mie=0)",
        unsafe { EXT0_FIRES } == 0,
    );
    check(
        &mut failures,
        "Ext0 stays pending",
        is_pending(Interrupt::Ext0),
    );

    // Phase 2: positive control — enabling mie lets the pending interrupt fire.
    let d = Deadline::start(50);
    enable_mie();
    d.spin_while(|| unsafe { EXT0_FIRES } == 0);
    check(
        &mut failures,
        "fires after enable_mie (positive control)",
        unsafe { EXT0_FIRES } == 1,
    );

    finish(failures, "clicwfi-01")
}

#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
}
