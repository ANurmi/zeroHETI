pub mod apb_timer;
pub mod clic;
pub mod hetic;
pub mod i2c;
pub mod mtimer;
pub mod uart;

/// Base address for Hetic/CLIC/EDFIC
pub const INTC_BASE: usize = 0x0010_0000;
