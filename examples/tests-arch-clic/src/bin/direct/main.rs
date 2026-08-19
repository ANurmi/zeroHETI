//! clicdirect-01 — direct-mode handler entry, trigger/clear, no retrigger of
//! the same interrupt while its level is not above `mintstatus.mil`.
//!
//!
//! # Status: EXPECTED-FAIL (known rt-ibex limitation)
//!
//! Taking a direct-mode (`shv=0`) CLIC interrupt glitches the rt-ibex
//! instruction fetch during the vector redirect: the core fetches a corrupted
//! copy of the vector-table entry (`0x995fe06c` instead of `0x995fe06f`), which
//! decodes as an illegal instruction. That exception (mcause `0x30000002`,
//! irq=0/cause=2) preempts the interrupt and lands in `Breakpoint`, so the test
//! hangs instead of completing.
//!
//! Reproduced with a bare `loop {}` and with a busy polling loop, and with the
//! pending bit set before and after `enable_mie()` — i.e. it is not a timing or
//! test artifact. The SHV path (`shv=1`, used by all other clic_arch tests) is
//! unaffected. Do not gate CI on this test until the rt-ibex issue is fixed.
#![no_main]
#![no_std]

use bsp::{
    clic::{Polarity, Trig, intattr::Mode},
    interrupt::Interrupt,
    register::{mcause, mepc},
    rt::entry,
    sprintln,
};

use tests_arch_clic::*;

/// Number of times the direct handler has run. Single-hart test, accessed only
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

/// The direct handler redirects `mepc` here, so this is where execution resumes
/// after the handler's `mret`.
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "handler ran exactly once",
        unsafe { EXT0_FIRES } == 1,
    );
    tests_arch_clic::finish(failures, "clicdirect-01")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    // Direct mode: shv=0 -> entry via mtvec base -> DefaultHandler.
    setup_irq_full(
        Interrupt::Ext0,
        1,
        Trig::Edge,
        Polarity::Pos,
        false,
        Mode::Machine,
        true,
    );
    set_thresh(0);
    disable_mie();

    sprintln!(
        "clicdirect-01: direct-mode entry (shv=0), trigger/clear, no retrigger while mil held"
    );

    enable_mie();
    pend(Interrupt::Ext0);

    // Never returns: the handler redirects mepc to `do_finish`.
    loop {}
}

/// Direct-mode handler, reached via `_default_start_trap` -> `_start_trap_rust`
/// -> `DefaultHandler`. The trap entry does not touch `mstatus`/`mepc`/`mcause`,
/// so the CSR writes below survive to the final `mret`.
#[unsafe(export_name = "DefaultHandler")]
unsafe extern "C" fn direct_handler() {
    unsafe {
        EXT0_FIRES += 1;
    }

    // Direct entry (shv=0) -> mcause: irq=1, code=27 (Ext0), minhv=0.
    let mc = mcause::read();
    hcheck("mcause.is_interrupt()", mc.is_interrupt());
    hcheck("mcause.code() == 27", mc.code() == 27);
    hcheck("mcause.minhv == 0", mc.bits() & (1 << 30) == 0);
    hcheck("mintstatus.mil == 1", read_mintstatus_mil() == 1);

    // Re-pend the same interrupt and re-enable mie. With mil still 1, level 1
    // is not above max(mil, thresh), so it must NOT re-enter the handler.
    pend(Interrupt::Ext0);
    enable_mie();
    let d = Deadline::start(50);
    d.spin_full();
    hcheck("no re-entry while mil=1", unsafe { EXT0_FIRES } == 1);

    // Clear the pending line so it cannot fire again after mret (mil drops back
    // to 0 on the mret `mstatus` restore).
    unpend(Interrupt::Ext0);

    // Redirect the return address to `do_finish`; the trap return is a plain
    // mret, so the CSR writes above survive.
    unsafe { mepc::write(do_finish as usize) };
}
