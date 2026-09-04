//! cliclevel-03 — a lower-level interrupt does not preempt a higher-level
//! handler.
//!
//! Ext0 (level 0x80) fires first; inside its handler Ext1 (level 0x10) is
//! asserted. Since 0x10 is not strictly above max(mintstatus.mil = 0x80,
//! mintthresh = 0), Ext1 must stay pending and never enter its handler.
#![no_main]
#![no_std]

use bsp::{interrupt::Interrupt, rt as riscv_rt, rt::entry, sprintln};

use tests_arch_clic::*;

/// Number of times each handler has run. Single-hart test, accessed only from
/// the handlers (and `do_finish` after the handlers' `mret`).
static mut EXT0_FIRES: u32 = 0;
static mut EXT1_FIRES: u32 = 0;
/// Handler-side failure counter (checked by `do_finish`).
static mut FAILURES: u32 = 0;

/// Handler-side check; accumulates into the global `FAILURES` counter.
fn hcheck(label: &str, cond: bool) {
    if cond {
        sprintln!("  [OK]   {label}");
    } else {
        sprintln!("  [FAIL] {label}");
        unsafe {
            FAILURES += 1;
        }
    }
}

/// The Ext0 handler redirects `mepc` here, so this is where execution resumes
/// after its `mret`.
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "Ext0 handler ran once",
        unsafe { EXT0_FIRES } == 1,
    );
    check(
        &mut failures,
        "Ext1 never fired (0x10 not > mil=0x80)",
        unsafe { EXT1_FIRES } == 0,
    );
    finish(failures, "cliclevel-03")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 0x80); // high level
    setup_irq_lvl(Interrupt::Ext1, 0x10); // low level
    set_thresh(0);
    enable_mie();

    sprintln!("cliclevel-03: lvl-1 (Ext1) does not preempt lvl-2 (Ext0) handler");

    // Never returns: the Ext0 handler redirects mepc to `do_finish`.
    pend(Interrupt::Ext0);
    loop {}
}

/// int1 (high level): runs first, asserts a lower-level int2 and gives it a
/// chance to (wrongly) preempt before finishing.
#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
    pend(Interrupt::Ext1);
    // 0x10 is not above max(mil=0x80, thresh=0) -> must not preempt.
    let d = Deadline::start(50);
    d.spin_full();
    hcheck("Ext1 not entered (0x10 not > mil=0x80)", unsafe { EXT1_FIRES } == 0);
    hcheck("Ext1 stays pending", is_pending(Interrupt::Ext1));
    // Clear it so it cannot fire after mret (mil drops back to 0).
    unpend(Interrupt::Ext1);
    unsafe { bsp::register::mepc::write(do_finish as usize) };
}

/// int2 (low level): must never run; defined only to satisfy the vector table
/// and to catch a wrong preemption.
#[bsp::core_interrupt(Interrupt::Ext1)]
fn ext1() {
    unsafe {
        EXT1_FIRES += 1;
    }
}