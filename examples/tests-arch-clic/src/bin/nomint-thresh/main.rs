//! TODO(phase 1): clicnomint-03 — no interrupt fires when its level is not
//! above `mintthresh`.
#![no_main]
#![no_std]


use bsp::rt::entry;

#[entry]
fn main() -> ! {
    // Stub: not implemented yet, fail loudly.
    bsp::tb::signal_fail(None);
    loop {}
}
