//! TODO(phase 1): clicdirect-01 — direct-mode handler entry, trigger/clear, no
//! retrigger of the same interrupt while its level is not above
//! `mintstatus.mil`.
#![no_main]
#![no_std]


use bsp::rt::entry;

#[entry]
fn main() -> ! {
    // Stub: not implemented yet, fail loudly.
    bsp::tb::signal_fail(None);
    loop {}
}
