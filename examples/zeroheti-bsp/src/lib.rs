#![no_std]

pub mod apb_uart;
#[cfg(feature = "core-fmt")]
mod core_sprint;
pub mod hetic;
pub mod mmap;
pub mod mmio;
#[cfg(feature = "panic")]
pub mod panic;
pub mod register;
pub mod tb;
pub mod trap;
#[cfg(feature = "ufmt")]
mod ufmt_sprint;

#[cfg(feature = "ufmt")]
pub use ufmt;

pub use embedded_io;
pub use riscv;
#[cfg(feature = "rt")]
pub use riscv_rt as rt;

use core::arch::asm;

#[cfg(not(any(feature = "fpga", feature = "rtl-tb")))]
compile_error!(
    "Select exactly one of -Ffpga -Frtl-tb, BSP supports FPGA and RTL testbench implementations only"
);

#[cfg(all(feature = "fpga", feature = "rtl-tb"))]
compile_error!("Select exactly one of -Ffpga -Frtl-tb");

#[cfg_attr(feature = "rtl-tb", doc = "100 MHz")]
pub const CPU_FREQ_HZ: u32 = match () {
    #[cfg(feature = "rtl-tb")]
    () => 100_000_000,
    #[cfg(feature = "fpga")]
    () => 25_000_000,
};

// Experimentally found value for how to adjust for real-time
const fn nop_mult() -> u32 {
    match () {
        #[cfg(debug_assertions)]
        () => 60 / 7,
        #[cfg(not(debug_assertions))]
        () => 60 / 8,
    }
}
pub const NOPS_PER_SEC: u32 = CPU_FREQ_HZ / nop_mult();

pub fn asm_delay(t: u32) {
    for _ in 0..t {
        unsafe { asm!("nop") }
    }
}
