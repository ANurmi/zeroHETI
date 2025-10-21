use crate::mmap::hetic::*;
use crate::mmio;

pub struct HeticIrqLine {
    idx: usize,
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
        mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx as usize, 0b1u8 << IP_OFS);
    }

    pub fn enable(&mut self) {
        mmio::mask_u8(HETIC_BASE + LINE_SIZE * self.idx as usize, 0b1u8 << IE_OFS);
    }
    pub fn disable(&mut self) {
        mmio::unmask_u8(HETIC_BASE + LINE_SIZE * self.idx as usize, 0b1u8 << IE_OFS);
    }
}

pub struct Hetic;

impl Hetic {
    pub fn line(idx: usize) -> HeticIrqLine {
        HeticIrqLine { idx }
    }
}
