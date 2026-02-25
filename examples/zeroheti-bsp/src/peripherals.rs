use crate::{i2c::I2c, mmap, mtimer::MTimer, timer_group::Timer};

/// Placeholder for RTIC
#[allow(unused)]
pub struct Peripherals {
    i2c: I2c,
    mtimer: MTimer,
    timer0: Timer,
    timer1: Timer,
    timer2: Timer,
    timer3: Timer,
}

impl Peripherals {
    pub unsafe fn steal() -> Self {
        Self {
            i2c: unsafe { I2c::instance() },
            mtimer: MTimer::instance(),
            timer0: Timer::init::<{ mmap::apb_timer::TIMER0_ADDR }>(),
            timer1: Timer::init::<{ mmap::apb_timer::TIMER1_ADDR }>(),
            timer2: Timer::init::<{ mmap::apb_timer::TIMER2_ADDR }>(),
            timer3: Timer::init::<{ mmap::apb_timer::TIMER3_ADDR }>(),
        }
    }
}
