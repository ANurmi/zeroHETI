//! smclicshv — selective hardware vectoring entry via `mtvt + 4*id`, with
//! `mcause.minhv = 1`, and edge `clicintip` auto-clear on take.
#![no_main]
#![no_std]

use bsp::{
    clic::{Polarity, Trig, intattr::Mode},
    interrupt::Interrupt,
    register::mcause,
    rt as riscv_rt,
    rt::entry,
    sprintln,
};

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

/// The SHV handler redirects `mepc` here, so this is where execution resumes
/// after the handler's `mret`.
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "handler ran exactly once",
        unsafe { EXT0_FIRES } == 1,
    );
    finish(failures, "smclicshv")
}

#[entry]
fn main() -> ! {
    init_arch_test();
    // SHV, edge, positive polarity, level 1, M-mode, enabled.
    setup_irq_full(
        Interrupt::Ext0,
        1,
        Trig::Edge,
        Polarity::Pos,
        true,
        Mode::Machine,
        true,
    );
    set_thresh(0);
    enable_mie();

    sprintln!("smclicshv: shv=1 entry via mtvt+4*id, minhv=1, edge ip auto-clear");

    // Never returns: the handler redirects mepc to `do_finish`.
    pend(Interrupt::Ext0);
    loop {}
}

/// SHV handler, reached by jumping to the address stored in the vector table
/// entry `mtvt + 4*27`. The SHV entry does not run the software trap entry, so
/// mcause/mepc/mstatus survive untouched to the final `mret`.
#[bsp::core_interrupt(Interrupt::Ext0)]
fn ext0() {
    unsafe {
        EXT0_FIRES += 1;
    }
    let mc = mcause::read();
    hcheck("mcause.is_interrupt()", mc.is_interrupt());
    hcheck("mcause.code() == 27", mc.code() == 27);
    hcheck("mcause.minhv == 1 (SHV)", mc.bits() & (1 << 30) != 0);
    hcheck("edge ip auto-cleared on take", !is_pending(Interrupt::Ext0));
    unsafe { bsp::register::mepc::write(do_finish as usize) };
}