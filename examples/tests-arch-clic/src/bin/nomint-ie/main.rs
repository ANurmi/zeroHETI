//! clicnomint-02 — no interrupt fires while `clicintie = 0`.
#![no_main]
#![no_std]

use bsp::{
    clic::{Polarity, Trig, intattr::Mode},
    interrupt::Interrupt,
    rt as riscv_rt,
    rt::entry,
    sprintln,
};

use tests_arch_clic::*;

/// Handler run counters. Single-hart test, accessed only from `main` and the
/// corresponding handlers.
static mut EXT0_FIRES: u32 = 0;
static mut EXT1_FIRES: u32 = 0;

#[entry]
fn main() -> ! {
    let mut failures = 0;

    init_arch_test();
    setup_irq_lvl(Interrupt::Ext1, 1);
    setup_irq_full(
        Interrupt::Ext0,
        1,
        Trig::Edge,
        Polarity::Pos,
        true,
        Mode::Machine,
        false, // ie = 0: Ext0 must NOT fire
    );

    sprintln!("clicnomint-02: no interrupt fires while clicintie=0");

    // Phase 1: mie=1, pend both; only Ext1 (ie=1) may fire. Ext0 (ie=0) must
    // stay pending and its handler must not be called.
    enable_mie();
    pend(Interrupt::Ext0);
    pend(Interrupt::Ext1);
    {
        let d = Deadline::start(50);
        d.spin_while(|| unsafe { EXT1_FIRES } == 0);
        d.spin_full();
    }
    check(
        &mut failures,
        "Ext1 fires (ie=1)",
        unsafe { EXT1_FIRES } == 1,
    );
    check(
        &mut failures,
        "Ext0 handler not called (ie=0)",
        unsafe { EXT0_FIRES } == 0,
    );
    check(
        &mut failures,
        "Ext0 stays pending (ie=0)",
        is_pending(Interrupt::Ext0),
    );
    check(
        &mut failures,
        "Ext1 pending cleared on take",
        !is_pending(Interrupt::Ext1),
    );

    // Phase 2: positive control — enabling Ext0's ie lets it fire.
    let d = Deadline::start(50);
    enable_ie(Interrupt::Ext0);
    d.spin_while(|| unsafe { EXT0_FIRES } == 0);
    check(
        &mut failures,
        "fires after enable_ie (positive control)",
        unsafe { EXT0_FIRES } == 1,
    );

    finish(failures, "clicnomint-02")
}

#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
}

#[bsp::core_interrupt(Interrupt::Ext1)]
fn ext1() {
    unsafe {
        EXT1_FIRES += 1;
    }
}
