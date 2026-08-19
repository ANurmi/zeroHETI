//! smclicmnxti — `mnxti` CSR reports the highest-priority pending interrupt
//! without claiming it.
//!
//! `mnxti` is a peek: it returns `{mtvt[31:8], id<<2}` for the pending
//! interrupt whose level is above `max(mil, thresh)` and whose `shv=0`
//! (with `CLIC_SHV=1` only non-SHV interrupts are reported), and 0 otherwise.
//! With `mstatus.mie = 0` the interrupt is never taken, so the peek is
//! side-effect free (rt-ibex implements `mnxti` without claim or jump).
#![no_main]
#![no_std]

use bsp::{
    clic::{Polarity, Trig, intattr::Mode},
    interrupt::Interrupt,
    register::mtvt,
    rt::entry,
    sprintln,
};

use tests_arch_clic::*;

/// Decode the interrupt id from an `mnxti` value.
fn mnxti_id(val: usize) -> usize {
    (val & 0xFF) >> 2
}

#[entry]
fn main() -> ! {
    let mut failures = 0;

    init_arch_test();
    // Non-SHV, edge, level 1, M-mode, enabled — level 1 > max(0, 0).
    setup_irq_full(Interrupt::Ext0, 1, Trig::Edge, Polarity::Pos, false, Mode::Machine, true);
    setup_irq_full(Interrupt::Ext1, 1, Trig::Edge, Polarity::Pos, false, Mode::Machine, true);
    set_thresh(0);
    // Peek with mie=0 so the (direct-mode) interrupts are never taken.
    disable_mie();

    let base = mtvt::read().address();
    sprintln!("smclicmnxti: mnxti peek reports top pending non-SHV interrupt (mie=0)");

    check(
        &mut failures,
        "mnxti == 0 with nothing pending",
        read_mnxti() == 0,
    );

    // Single pending interrupt.
    pend(Interrupt::Ext0);
    let v0 = read_mnxti();
    sprintln!("  mnxti with Ext0 pending: 0x{v0:08x}");
    check(
        &mut failures,
        "mnxti encodes Ext0 (27)",
        v0 != 0 && mnxti_id(v0) == 27,
    );
    check(
        &mut failures,
        "mnxti returns the mtvt base",
        v0 & !0xFF == base,
    );

    // Two pending at equal level: on a tie the CLIC's tree keeps the
    // lowest-indexed source (Ext0 = 27).
    pend(Interrupt::Ext1);
    let v1 = read_mnxti();
    sprintln!("  mnxti with Ext0+Ext1 pending: 0x{v1:08x}");
    check(
        &mut failures,
        "tie at level 1 resolved to Ext0 (27)",
        v1 != 0 && mnxti_id(v1) == 27,
    );
    check(
        &mut failures,
        "mnxti still returns the mtvt base",
        v1 & !0xFF == base,
    );

    // Note: the two pended edges were never claimed, and on this CLIC an
    // unclaimed edge pending is only cleared by a claim, not by unpend — so
    // there is no clean way to return to "nothing pending" here. The test
    // ends with both lines latched pending, which is fine.

    finish(failures, "smclicmnxti")
}