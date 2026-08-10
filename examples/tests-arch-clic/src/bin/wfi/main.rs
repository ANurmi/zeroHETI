//! TODO(phase 1): clicwfi-01 — `wfi` behaves like a nop and wakes without
//! trapping when an interrupt is pending and `mstatus.mie = 0`.
#![no_main]
#![no_std]


use bsp::rt::entry;

#[entry]
fn main() -> ! {
    // Stub: not implemented yet, fail loudly.
    bsp::tb::signal_fail(None);
    loop {}
}
