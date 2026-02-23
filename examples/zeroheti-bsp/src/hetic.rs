use crate::mmap::{INTC_BASE, hetic::*};
use crate::mmio;

// Re-export useful riscv-pac traits
pub use riscv_pac::{HartIdNumber, PriorityNumber};

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
        mmio::write_u8(INTC_BASE + LINE_SIZE * self.idx + 1, level);
    }
    pub fn level(&self) {
        mmio::read_u8(INTC_BASE + LINE_SIZE * self.idx + 1);
    }

    pub fn pend(&mut self) {
        mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << IP_BIT);
    }
    pub fn unpend(&mut self) {
        mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << IP_BIT);
    }

    pub fn enable(&mut self) {
        mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << IE_BIT);
    }
    pub fn disable(&mut self) {
        mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << IE_BIT);
    }

    pub fn set_heti(&mut self) {
        mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << HETI_BIT);
    }
    pub fn unset_heti(&mut self) {
        mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << HETI_BIT);
    }

    pub fn set_nest(&mut self) {
        mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << NEST_BIT);
    }
    pub fn unset_nest(&mut self) {
        mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << NEST_BIT);
    }

    /// Set the priority. It's not the level.
    pub fn set_prio(&mut self, prio: u8) {
        mmio::write_u8(INTC_BASE + LINE_SIZE * self.idx + PRIO_OFS, prio);
    }

    /// Set trigger type (edge/level)
    pub fn set_trig(&mut self, trig: Trig) {
        match trig {
            Trig::Edge => mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << TRIG_BIT),
            Trig::Level => mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << TRIG_BIT),
        }
    }

    /// Set polarity (positive/negative)
    pub fn set_pol(&mut self, pol: Pol) {
        match pol {
            Pol::Pos => mmio::unmask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << POL_BIT),
            Pol::Neg => mmio::mask_u8(INTC_BASE + LINE_SIZE * self.idx, 0b1u8 << POL_BIT),
        }
    }
}

pub struct Hetic;

impl Hetic {
    pub fn line(idx: usize) -> HeticIrqLine {
        HeticIrqLine { idx }
    }
}
