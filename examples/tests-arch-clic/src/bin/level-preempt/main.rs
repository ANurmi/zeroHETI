//! cliclevel-02 — a higher-level interrupt preempts a lower-level handler.
//!
//! Ext0 (level 0x10) fires first; inside its handler Ext1 (level 0x80) is
//! asserted. Since 0x80 > max(mintstatus.mil = 0x10, mintthresh = 0), the CLIC
//! preempts into the Ext1 handler, which records its `mcause` and
//! `mintstatus.mil`, and on `mret` execution resumes inside the Ext0 handler.
#![no_main]
#![no_std]

use bsp::{
    interrupt::Interrupt,
    register::mcause,
    rt as riscv_rt,
    rt::entry,
    sprintln,
};

use tests_arch_clic::*;

/// Number of times each handler has run. Single-hart test, accessed only from
/// the handlers (and `do_finish` after the handlers' `mret`).
static mut EXT0_FIRES: u32 = 0;
static mut EXT1_FIRES: u32 = 0;
/// Visit order of the two handlers, recorded as interrupt numbers (27, 28).
static mut VISIT: [u32; 2] = [0; 2];
static mut VISIT_COUNT: u32 = 0;
/// Ext1-handler observations, checked by `do_finish`.
static mut EXT1_MCAUSE_CODE: u32 = 0;
static mut EXT1_MIL: u32 = 0;
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
/// after its `mret` (the preempting Ext1 handler has already returned).
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(&mut failures, "Ext0 handler ran", unsafe { EXT0_FIRES } == 1);
    check(&mut failures, "Ext1 handler ran", unsafe { EXT1_FIRES } == 1);
    check(
        &mut failures,
        "Ext1 mcause.code == 28",
        unsafe { EXT1_MCAUSE_CODE } == 28,
    );
    check(
        &mut failures,
        "Ext1 mintstatus.mil == 0x80",
        unsafe { EXT1_MIL } == 0x80,
    );
    check(
        &mut failures,
        "visit order [Ext0, Ext1]",
        unsafe { VISIT[0] } == 27 && unsafe { VISIT[1] } == 28,
    );
    finish(failures, "cliclevel-02")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 0x10); // low level
    setup_irq_lvl(Interrupt::Ext1, 0x80); // high level
    set_thresh(0);
    enable_mie();

    sprintln!("cliclevel-02: lvl-2 (Ext1) preempts lvl-1 (Ext0) handler");

    // Never returns: the Ext0 handler redirects mepc to `do_finish`.
    pend(Interrupt::Ext0);
    loop {}
}

/// int1 (low level): runs first, asserts int2, and waits for the preemption
/// and return before finishing.
#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
        VISIT[VISIT_COUNT as usize] = 27;
        VISIT_COUNT += 1;
    }
    // Assert int2 while this handler is active (mil = 0x10). The CLIC leaves
    // mstatus.mie untouched on the trap entry, so Ext1 (level 0x80) is taken
    // as soon as it is pending.
    pend(Interrupt::Ext1);
    let d = Deadline::start(50);
    d.spin_while(|| unsafe { EXT1_FIRES } == 0);
    hcheck("Ext1 preempted and returned", unsafe { EXT1_FIRES } == 1);
    // Execution is back in this handler: redirect the return to `do_finish`.
    unsafe { bsp::register::mepc::write(do_finish as usize) };
}

/// int2 (high level): preempts `ext0`. The `nested_interrupt` trampoline
/// enables `mie` on entry so further preemption is possible.
#[bsp::nested_interrupt]
#[allow(non_snake_case)]
fn Ext1() {
    unsafe {
        EXT1_FIRES += 1;
        VISIT[VISIT_COUNT as usize] = 28;
        VISIT_COUNT += 1;
        EXT1_MCAUSE_CODE = mcause::read().code() as u32;
        EXT1_MIL = read_mintstatus_mil() as u32;
    }
}