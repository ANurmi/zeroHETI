use crate::mmap::hetic::*;
use crate::mmio;

// Re-export useful riscv-pac traits
pub use riscv_pac::{HartIdNumber, InterruptNumber, PriorityNumber};

pub struct HeticIrqLine {
    idx: usize,
}

#[derive(Clone, Copy)]
pub enum Trig {
    Edge = 0,
    Level = 1,
}

#[derive(Clone, Copy)]
pub enum Pol {
    /// Positive
    Pos = 0,
    /// Negative
    Neg = 1,
}

impl HeticIrqLine {
    pub fn set_level(&mut self, level: u8) {
        unsafe { mmio::write_u8(HETIC_BASE + LINE_SIZE * self.idx + 1, level) };
    }
    pub fn level(&self) {
        unsafe { mmio::read_u8(HETIC_BASE + LINE_SIZE * self.idx + 1) };
    }

    pub fn pend(&mut self) {
        unsafe { mmio::write_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << IP_OFS) };
    }
    pub fn unpend(&mut self) {
        mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << IP_OFS);
    }

    pub fn enable(&mut self) {
        mmio::mask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << IE_OFS);
    }
    pub fn disable(&mut self) {
        mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << IE_OFS);
    }

    /// Set trigger type (edge/level)
    pub fn set_trig(&mut self, trig: Trig) {
        match trig {
            Trig::Edge => mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << TRIG_OFS),
            Trig::Level => mmio::mask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << TRIG_OFS),
        }
    }

    /// Set polarity (positive/negative)
    pub fn set_pol(&mut self, pol: Pol) {
        match pol {
            Pol::Pos => mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << POL_OFS),
            Pol::Neg => mmio::mask_u8(HETIC_BASE + LINE_SIZE * self.idx, 0b1u8 << POL_OFS),
        }
    }
}

pub struct Hetic;

impl Hetic {
    pub fn line(idx: usize) -> HeticIrqLine {
        HeticIrqLine { idx }
    }
}
