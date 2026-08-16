//! smclicshv-illegal — an SHV vector-table entry pointing at an illegal
//! instruction raises an illegal-instruction exception instead of running the
//! handler.
//!
//! `mtvt` is repointed at a 256-byte-aligned table whose Ext0 entry (27) holds
//! the address of a word containing the illegal encoding `0xFFFFFFFF`. Taking
//! the SHV interrupt jumps there; executing `0xFFFFFFFF` traps with
//! `mcause` = illegal instruction (irq=0, code=2), which is routed to the
//! `IllegalInstruction` exception handler.
#![no_main]
#![no_std]

use bsp::{
    clic::{Polarity, Trig, intattr::Mode},
    interrupt::Interrupt,
    register::{mcause, mepc, mtvt},
    rt as riscv_rt,
    rt::entry,
    sprintln,
};

use tests_arch_clic::*;

/// SHV vector table. Entry [27] (Ext0) is patched to point at `ILLEGAL_WORD`.
/// It must be 256-byte aligned because `mtvt` only keeps bits [31:8].
#[repr(align(256))]
struct AlignedTable([u32; 32]);

static mut ILLEGAL_VTABLE: AlignedTable = AlignedTable([0; 32]);

/// A data word with an illegal RV32 encoding (`0xFFFFFFFF`: 32-bit, reserved
/// opcode 0x7F). Static data lives in DMEM, which is executable.
static mut ILLEGAL_WORD: u32 = 0xFFFF_FFFF;

/// Number of times the exception handler has run.
static mut EXC_FIRES: u32 = 0;
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

/// The exception handler redirects `mepc` here, so this is where execution
/// resumes after the trap entry's `mret`.
fn do_finish() -> ! {
    let mut failures = unsafe { FAILURES };
    check(
        &mut failures,
        "illegal-instruction exception handler ran",
        unsafe { EXC_FIRES } == 1,
    );
    finish(failures, "smclicshv-illegal")
}

#[entry]
fn main() -> ! {
    init_arch_test();
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

    // Repoint mtvt at a table whose Ext0 entry leads to an illegal instruction.
    unsafe {
        ILLEGAL_VTABLE.0[27] = &raw const ILLEGAL_WORD as usize as u32;
        mtvt::write(&raw const ILLEGAL_VTABLE as usize, mtvt::TrapMode::Clic);
    }

    sprintln!("smclicshv-illegal: SHV entry to illegal instruction traps (mcause=2), handler not run");

    // Never returns: the exception handler redirects mepc to `do_finish`.
    enable_mie();
    pend(Interrupt::Ext0);
    loop {}
}

/// Illegal-instruction exception handler, reached via `_start_trap` ->
/// `_default_start_trap` -> `_start_trap_rust` -> `_dispatch_exception` ->
/// `IllegalInstruction`. The trap entry does not touch `mepc`, so the CSR
/// write below survives to the final `mret`.
#[unsafe(export_name = "IllegalInstruction")]
unsafe extern "C" fn illegal_instruction(_tf: &riscv_rt::TrapFrame) {
    unsafe {
        EXC_FIRES += 1;
    }
    let mc = mcause::read();
    hcheck("mcause not an interrupt (exception)", !mc.is_interrupt());
    hcheck("mcause.code == 2 (illegal instruction)", mc.code() == 2);
    unsafe { mepc::write(do_finish as usize) };
}