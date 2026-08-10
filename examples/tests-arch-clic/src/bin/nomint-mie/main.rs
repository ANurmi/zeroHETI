//! TODO(phase 1): clicnomint-01 — no interrupt fires while `mstatus.mie = 0`.
#![no_main]
#![no_std]


use bsp::rt::entry;

#[entry]
fn main() -> ! {
    // Stub: not implemented yet, fail loudly.
    bsp::tb::signal_fail(None);
    loop {}
}
