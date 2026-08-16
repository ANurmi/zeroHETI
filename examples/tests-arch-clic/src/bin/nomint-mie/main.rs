//! clicnomint-01 — no interrupt fires while `mstatus.mie = 0`.
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

    sprintln!("clicnomint-01: no interrupt fires while mstatus.mie=0");

    // Phase 1: mie=0, pend Ext0, wait a bounded window. The interrupt must NOT
    // be taken and the pending bit must stay asserted.
    pend(Interrupt::Ext0);
    {
        let d = Deadline::start(50);
        d.spin_full();
    }
    check(
        &mut failures,
        "handler not called while mie=0",
        unsafe { EXT0_FIRES } == 0,
    );
    check(
        &mut failures,
        "Ext0 stays pending while mie=0",
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

    finish(failures, "clicnomint-01")
}

#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
}
