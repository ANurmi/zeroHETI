//! Memory maps for mtimer (RISC-V base ISA)
//!
//! mtimer is implemented by:
//!
//! - [timer_core (SV)](.../src/ip/timer_core.sv)
//! - [apb_mtimer (SV)](.../src/ip/apb_mtimer.sv)
//!
//! See also [crate::mmap::timer_group] for the general purpose timer group.
pub const MTIMER_BASE: usize = 0x3100;

pub const MTIME_LOW_ADDR_OFS: usize = 0;
pub const MTIME_HIGH_ADDR_OFS: usize = 4;
pub const MTIMECMP_LOW_ADDR_OFS: usize = 8;
pub const MTIMECMP_HIGH_ADDR_OFS: usize = 12;
pub const MTIME_CTRL_ADDR_OFS: usize = 16;

/// Prescaler field start bit in MTIME_CTRL (bits [17:8]).
pub const MTIME_CTRL_PS_SBIT: u32 = 8;
/// Number of bits reserved for the prescaler in `mtime_ctrl`
pub const MTIME_CTRL_PS_BITS: u32 = 10;
