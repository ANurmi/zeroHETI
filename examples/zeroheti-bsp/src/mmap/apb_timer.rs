pub const APB_TIMER_BASE: usize = 0x0_3400;

/// Timer address separation in memory layout
pub const TIMER_SEP: usize = 0x10;
pub const TIMER0_ADDR: usize = APB_TIMER_BASE;
pub const TIMER1_ADDR: usize = APB_TIMER_BASE + 1 * TIMER_SEP;
pub const TIMER2_ADDR: usize = APB_TIMER_BASE + 2 * TIMER_SEP;
pub const TIMER3_ADDR: usize = APB_TIMER_BASE + 3 * TIMER_SEP;
pub const TIMER4_ADDR: usize = APB_TIMER_BASE + 4 * TIMER_SEP;
pub const TIMER5_ADDR: usize = APB_TIMER_BASE + 5 * TIMER_SEP;
pub const TIMER6_ADDR: usize = APB_TIMER_BASE + 6 * TIMER_SEP;
pub const TIMER7_ADDR: usize = APB_TIMER_BASE + 7 * TIMER_SEP;
pub const TIMER8_ADDR: usize = APB_TIMER_BASE + 8 * TIMER_SEP;
pub const TIMER9_ADDR: usize = APB_TIMER_BASE + 9 * TIMER_SEP;
pub const TIMER10_ADDR: usize = APB_TIMER_BASE + 10 * TIMER_SEP;
pub const TIMER11_ADDR: usize = APB_TIMER_BASE + 11 * TIMER_SEP;
pub const TIMER12_ADDR: usize = APB_TIMER_BASE + 12 * TIMER_SEP;
pub const TIMER13_ADDR: usize = APB_TIMER_BASE + 13 * TIMER_SEP;
pub const TIMER14_ADDR: usize = APB_TIMER_BASE + 14 * TIMER_SEP;
pub const TIMER15_ADDR: usize = APB_TIMER_BASE + 15 * TIMER_SEP;

/// Timer counter
pub const TIMER_COUNTER_OFS: usize = 0x0;

/// Timer control
///
/// * `[0]` - enable
/// * `[3:5]` - prescaler
///
/// Hardware outputs value divided by prescaler
pub const TIMER_CTRL_OFS: usize = 0x4;
pub const TIMER_CTRL_ENABLE_BIT: u32 = 0b1;
pub const TIMER_CTRL_PRESCALER_BIT_IDX: u32 = 3;

/// Timer compare
///
/// Interrupt signal is raised on `timer >= timer_cmp`.
pub const TIMER_CMP_OFS: usize = 0x8;

#[repr(C)]
pub struct RegisterBlock {
    /// 0x0..0x4 Counter
    pub cnt: u32,
    /// 0x4..0x8 Control
    pub ctrl: u32,
    /// 0x8..0xc Compare
    pub cmp: u32,
}

pub const TIMER0: *mut RegisterBlock = TIMER0_ADDR as *mut _;
pub const TIMER1: *mut RegisterBlock = TIMER1_ADDR as *mut _;
pub const TIMER2: *mut RegisterBlock = TIMER2_ADDR as *mut _;
pub const TIMER3: *mut RegisterBlock = TIMER3_ADDR as *mut _;
