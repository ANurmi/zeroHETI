use riscv::InterruptNumber;

use crate::{
    timer_group::Duration,
    mmap::timer_queue::*,
    mmio,
};

pub struct TimerQueueHal<const BASE_ADDR: usize>;

pub type TimerQueue = TimerQueueHal<TQ_BASE>;

impl<const BASE_ADDR: usize> TimerQueueHal<BASE_ADDR> {
    pub fn init() -> Self {
        Self
    }

    pub fn instance() -> Self {
        Self {}
    }

    pub fn push_rel(&self, irq: impl InterruptNumber, timestamp: Duration) -> u8 {
        mmio::write_u32(BASE_ADDR, timestamp.as_ticks());
        0
    }
    pub fn push_abs(&self, irq: impl InterruptNumber, timestamp: Duration) -> u8 {
        0
    }
}
