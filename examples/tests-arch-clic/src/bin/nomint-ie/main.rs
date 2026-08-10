//! TODO(phase 1): clicnomint-02 — no interrupt fires while `clicintie = 0`.
#![no_main]
#![no_std]


use bsp::rt::entry;

#[entry]
fn main() -> ! {
    // Stub: not implemented yet, fail loudly.
    bsp::tb::signal_fail(None);
    loop {}
}
