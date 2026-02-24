use crate::mmap::{INTC_BASE, edfic::*};
use crate::mmio;

// Re-export useful riscv-pac traits
pub use riscv_pac::{HartIdNumber, PriorityNumber};

pub struct EdficIrqLine {
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

impl EdficIrqLine {
    pub fn pend(&mut self) {
        mmio::mask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << IP_BIT);
    }
    pub fn unpend(&mut self) {
        mmio::unmask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << IP_BIT);
    }

    pub fn enable(&mut self) {
        mmio::mask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << IE_BIT);
    }
    pub fn disable(&mut self) {
        mmio::unmask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << IE_BIT);
    }

    /// Set trigger type (edge/level)
    pub fn set_trig(&mut self, trig: Trig) {
        match trig {
            Trig::Edge => mmio::unmask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << TRIG_BIT),
            Trig::Level => mmio::mask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << TRIG_BIT),
        }
    }

    /// Set polarity (positive/negative)
    pub fn set_pol(&mut self, pol: Pol) {
        match pol {
            Pol::Pos => mmio::unmask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << POL_BIT),
            Pol::Neg => mmio::mask_u32(INTC_BASE + LINE_SIZE * self.idx, 0b1 << POL_BIT),
        }
    }

    /// Retrieve the 12--24-bit deadline
    pub fn dl(&self) -> u32 {
        (mmio::read_u32(INTC_BASE + LINE_SIZE * self.idx) & 0xffff_ff00) >> 8
    }

    /// Set the 12--24-bit deadline
    pub fn set_dl(&self, dl: u32) {
        mmio::mask_u32(INTC_BASE + LINE_SIZE * self.idx, (dl << 8) & 0xffff_ff00);
    }
}

pub struct Edfic;

impl Edfic {
    pub fn line(idx: usize) -> EdficIrqLine {
        EdficIrqLine { idx }
    }
}
