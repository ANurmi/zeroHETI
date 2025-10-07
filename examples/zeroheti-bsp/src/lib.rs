#![no_std]

pub mod apb_uart;
#[cfg(feature = "core-fmt")]
mod core_sprint;
pub mod mmap;
pub mod mmio;
#[cfg(feature = "panic")]
pub mod panic;
pub mod register;
pub mod tb;
#[cfg(feature = "ufmt")]
mod ufmt_sprint;

#[cfg(feature = "ufmt")]
pub use ufmt;

pub use riscv;
#[cfg(feature = "rt")]
pub use riscv_rt as rt;

use core::arch::asm;

#[cfg_attr(feature = "rtl-tb", doc = "100 MHz")]
pub const CPU_FREQ_HZ: u32 = match () {
    #[cfg(feature = "rtl-tb")]
    () => 100_000_000,
};

// Experimentally found value for how to adjust for real-time
const fn nop_mult() -> u32 {
    match () {
        #[cfg(debug_assertions)]
        () => 60 / 12,
        #[cfg(not(debug_assertions))]
        () => 60 / 13,
    }
}
pub const NOPS_PER_SEC: u32 = CPU_FREQ_HZ / nop_mult();

pub fn asm_delay(t: u32) {
    for _ in 0..t {
        unsafe { asm!("nop") }
    }
}
