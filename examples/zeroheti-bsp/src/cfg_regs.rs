use crate::{mmap::cfg_regs::*, mmio};

pub struct CfgRegsHal<const BASE_ADDR: usize>;

pub type CfgRegs = CfgRegsHal<CFG_BASE_ADDR>;

impl<const BASE_ADDR: usize> CfgRegsHal<BASE_ADDR> {
    pub fn init() -> Self {
        if !Self.intc_valid() {
            panic!("Interrupt controller mismatch! Recompile program or hardware.");
        }
        Self
    }

    pub const fn instance() -> Self {
        Self {}
    }

    #[inline]
    pub fn commit(&self) -> u32 {
        mmio::read_u32(CFG_BASE_ADDR)
    }

    #[inline]
    pub fn intc_edfic(&self) -> bool {
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        (cfg & 0b1) == 0b1
    }

    /// Check feature flags against HW-reported configuration
    #[inline]
    pub fn intc_valid(&self) -> bool {
        #[cfg(feature = "intc-edfic")]
        if !self.intc_edfic() {
            return false;
        }
        #[cfg(feature = "intc-clic")]
        if self.intc_edfic() {
            return false;
        }
        true
    }

    #[inline]
    pub fn full_uart(&self) -> bool {
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        (cfg & 0b10) == 0b10
    }

    #[inline]
    pub fn imem_bytes(&self) -> u32 {
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        let bytes = 2u32.pow((cfg & 0x00FF00) >> 8);
        bytes
    }

    #[inline]
    pub fn dmem_bytes(&self) -> u32 {
        let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
        let bytes = 2u32.pow((cfg & 0xFF0000) >> 16);
        bytes
    }
}
