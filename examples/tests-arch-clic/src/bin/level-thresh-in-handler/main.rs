//! cliclevel-04 — an interrupt whose level equals the raised `mintthresh` does
//! not preempt.
//!
//! Ext0 (level 0x10) fires first; its handler raises `mintthresh` to 0x80 and
//! asserts Ext1 (level 0x80). Preemption requires strictly `level >
//! max(mil, mintthresh)`, so 0x80 == thresh must not preempt. Positive control:
//! lowering the threshold to 0x40 lets the same Ext1 preempt.
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
/// after its `mret` (the preempting Ext1 handler has already returned).
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "Ext0 handler ran once",
        unsafe { EXT0_FIRES } == 1,
    );
    check(
        &mut failures,
        "Ext1 ran exactly once (positive control)",
        unsafe { EXT1_FIRES } == 1,
    );
    finish(failures, "cliclevel-04")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 0x10); // low level
    setup_irq_lvl(Interrupt::Ext1, 0x80); // level equal to the raised thresh
    set_thresh(0);
    enable_mie();

    sprintln!("cliclevel-04: int level equal to mintthresh does not preempt");

    // Never returns: the Ext0 handler redirects mepc to `do_finish`.
    pend(Interrupt::Ext0);
    loop {}
}

/// int1 (low level): runs first, raises the threshold, and asserts int2.
#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
    // Raise the threshold to exactly Ext1's level while this handler is active.
    set_thresh(0x80);
    pend(Interrupt::Ext1);
    // 0x80 is not strictly above max(mil=0x10, thresh=0x80) -> must not preempt.
    let d = Deadline::start(50);
    d.spin_full();
    hcheck("Ext1 not preempted (0x80 == thresh)", unsafe { EXT1_FIRES } == 0);
    hcheck("Ext1 stays pending", is_pending(Interrupt::Ext1));

    // Positive control: 0x80 > max(0x10, 0x40) -> Ext1 preempts now.
    set_thresh(0x40);
    let d = Deadline::start(50);
    d.spin_while(|| unsafe { EXT1_FIRES } == 0);
    hcheck(
        "Ext1 preempts after lowering thresh (positive control)",
        unsafe { EXT1_FIRES } == 1,
    );

    // Execution is back in this handler: redirect the return to `do_finish`.
    unsafe { bsp::register::mepc::write(do_finish as usize) };
}

/// int2 (max level): preempts `ext0` only once the threshold is lowered.
#[bsp::nested_interrupt]
#[allow(non_snake_case)]
fn Ext1() {
    unsafe {
        EXT1_FIRES += 1;
    }
}