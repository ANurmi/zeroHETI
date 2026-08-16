//! clicnomint-03 — no interrupt fires when its level is not above `mintthresh`.
#![no_main]
#![no_std]

use bsp::{interrupt::Interrupt, rt as riscv_rt, rt::entry, sprintln};

use tests_arch_clic::*;

/// Handler run counters. Single-hart test, accessed only from `main` and the
/// corresponding handlers.
static mut EXT0_FIRES: u32 = 0;
static mut EXT1_FIRES: u32 = 0;

#[entry]
fn main() -> ! {
    let mut failures = 0;

    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 0x10);
    setup_irq_lvl(Interrupt::Ext1, 0x80);
    set_thresh(0x40);

    sprintln!("clicnomint-03: no interrupt fires when level is not above mintthresh");

    // Phase 1: mie=1, pend both. Only Ext1 (lvl 0x80 > thresh 0x40) may fire;
    // Ext0 (lvl 0x10 <= thresh 0x40) must stay pending.
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
        "Ext1 fires (lvl 0x80 > thresh 0x40)",
        unsafe { EXT1_FIRES } == 1,
    );
    check(
        &mut failures,
        "Ext0 handler not called (lvl 0x10 <= thresh 0x40)",
        unsafe { EXT0_FIRES } == 0,
    );
    check(
        &mut failures,
        "Ext0 stays pending",
        is_pending(Interrupt::Ext0),
    );
    check(
        &mut failures,
        "Ext1 pending cleared on take",
        !is_pending(Interrupt::Ext1),
    );

    // Phase 2: positive control — lowering the threshold below Ext0's level
    // lets the still-pending Ext0 fire.
    let d = Deadline::start(50);
    set_thresh(0x00);
    d.spin_while(|| unsafe { EXT0_FIRES } == 0);
    check(
        &mut failures,
        "fires after thresh lowered (positive control)",
        unsafe { EXT0_FIRES } == 1,
    );

    finish(failures, "clicnomint-03")
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
