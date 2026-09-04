//! edge-level — edge-triggered SW pending stays pending (not auto-cleared)
//! while `mil` is held, and a fresh edge re-fires only after `mret`.
//!
//! The edge `clicintip` pending bit is cleared on core acknowledge (claim), not
//! by level. Re-pending Ext0 inside its own handler (a fresh edge) must not
//! re-enter while `mil` is still 1; after the handler's `mret` (mil back to 0)
//! the still-pending edge is taken again.
//!
//! # Coverage hole: level-triggered part
//!
//! Level-triggered interrupts are NOT testable via software on this part:
//! `clic_gateway` level mode (`le = 0`) drives `ip` straight from the external
//! source line and ignores the software `clicintip` writes, and the verilated
//! top ties all external interrupt lines off except I2C
//! (`verilator/tb/zeroheti_top_wrapper.sv`). Only the edge side is exercised.
#![no_main]
#![no_std]

use bsp::{interrupt::Interrupt, rt as riscv_rt, rt::entry, sprintln};

use tests_arch_clic::*;

/// Number of times the Ext0 handler has run. Single-hart test, accessed only
/// from the handler (and `do_finish` after the handler's `mret`).
static mut EXT0_FIRES: u32 = 0;
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

/// Where execution resumes after the first handler's `mret`.
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "first entry happened (sanity)",
        unsafe { EXT0_FIRES } >= 1,
    );
    // The pending edge may already have re-fired at the `mret` boundary
    // (mil drops back to 0 while `mie` is still set); if not, enable `mie`
    // and wait for it.
    enable_mie();
    let d = Deadline::start(50);
    d.spin_while(|| unsafe { EXT0_FIRES } < 2);
    check(
        &mut failures,
        "edge re-pend re-fires after mret",
        unsafe { EXT0_FIRES } == 2,
    );
    finish(failures, "edge-level")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    setup_irq_lvl(Interrupt::Ext0, 1); // SHV, edge, level 1
    set_thresh(0);
    enable_mie();

    sprintln!("edge-level: edge re-pend stays pending while mil=1, re-fires after mret");

    // Never returns: the first handler entry redirects mepc to `do_finish`.
    pend(Interrupt::Ext0);
    loop {}
}

/// SHV handler, reached by jumping to the address stored in the vector table
/// entry `mtvt + 4*27`.
#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
    // First entry only: the claim cleared the edge ip on take, so pend a fresh
    // edge while this handler is active (mil = 1). It must latch pending
    // without re-entering, and be taken only after `mret` drops mil to 0.
    if unsafe { EXT0_FIRES } == 1 {
        pend(Interrupt::Ext0);
        let d = Deadline::start(50);
        d.spin_full();
        hcheck("no re-entry while mil=1", unsafe { EXT0_FIRES } == 1);
        hcheck("edge re-pend is pending", is_pending(Interrupt::Ext0));
        // Redirect: after `mret` the still-pending edge must re-fire.
        unsafe { bsp::register::mepc::write(do_finish as usize) };
    }
    // Second entry: plain return (`mret`) back to do_finish.
}