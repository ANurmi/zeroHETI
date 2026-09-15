pub mod apb_timer;
pub mod clic;
pub mod edfic;
pub mod timer_queue;
pub mod i2c;
pub mod cfg_regs;
pub mod mailbox;
pub mod mtimer;
pub mod uart;

/// Base address for CLIC/EDFIC
pub const INTC_BASE: usize = 0x0010_0000;
