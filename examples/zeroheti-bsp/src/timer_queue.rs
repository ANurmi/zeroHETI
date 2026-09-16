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
        // TODO: flush out queue
        Self
    }

    pub fn instance() -> Self {
        Self {}
    }

    #[inline]
    pub fn push_rel(&self, irq: impl InterruptNumber, timestamp: Duration) {
        mmio::write_u32(BASE_ADDR+REL_TS_OFFS, timestamp.as_ticks());
        let cmd: u32 = 0x1 | (((irq.number() as u32)-32) << 24);
        mmio::write_u32(BASE_ADDR+CTRL_OFFS, cmd);
    }

    #[inline]
    pub fn push_abs(&self, irq: impl InterruptNumber, timestamp: u64)  {
        mmio::write_u32(BASE_ADDR+ABS_TS_LO_OFFS, timestamp as u32);
        mmio::write_u32(BASE_ADDR+ABS_TS_HI_OFFS, (timestamp >> 32) as u32);
        let cmd: u32 = 0x2 | (((irq.number() as u32)-32) << 24);
        mmio::write_u32(BASE_ADDR+CTRL_OFFS, cmd);
    }
}
