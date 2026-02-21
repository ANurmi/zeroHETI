use riscv::CoreInterruptNumber;
use riscv_pac::{ExternalInterruptNumber, InterruptNumber};
use strum::FromRepr;

#[derive(Clone, Copy, PartialEq, FromRepr)]
#[repr(usize)]
#[cfg_attr(not(feature = "ufmt"), derive(Debug))]
pub enum ExternalInterrupt {
    /// Timer 0 overflow
    Timer0Ovf = 16,
    /// Timer 0 compare
    Timer0Cmp = 17,
    /// Timer1 overflow
    Timer1Ovf = 18,
    /// Timer1 compare
    Timer1Cmp = 19,
    /// Timer2 overflow
    Timer2Ovf = 20,
    /// Timer2 compare
    Timer2Cmp = 21,
    /// Timer3 overflow
    Timer3Ovf = 22,
    /// Timer3 compare
    Timer3Cmp = 23,
    Uart = 24,
    I2c = 25,
    /// Mailbox
    Mbx = 26,
    /// Generic external interrupt 0
    Ext0 = 27,
    /// Generic external interrupt 1
    Ext1 = 28,
    /// Generic external interrupt 2
    Ext2 = 29,
    /// Generic external interrupt 3
    Ext3 = 30,
    /// Non-maskable interrupt, carried over from standard Ibex
    ///
    /// ???: Nmi doesn't seem to be working on zeroHETI (nor did it work on
    /// Atalanta)
    Nmi = 31,
}

unsafe impl ExternalInterruptNumber for ExternalInterrupt {}

unsafe impl InterruptNumber for ExternalInterrupt {
    const MAX_INTERRUPT_NUMBER: usize = Self::Nmi as usize;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> Result<Self, riscv_pac::result::Error> {
        Self::from_repr(value).ok_or(riscv_pac::result::Error::InvalidVariant(value))
    }
}

/// Standard M-mode RISC-V interrupts
///
/// Should be used in place of riscv::Interrupt
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum CoreInterrupt {
    // NC: SupervisorSoft = 1,
    MachineSoft = 3,
    // NC: SupervisorTimer = 5,
    MachineTimer = 7,
    // NC: SupervisorExternal = 9,
    MachineExternal = 11,
}

/// SAFETY: `Interrupt` represents the standard RISC-V interrupts
unsafe impl InterruptNumber for CoreInterrupt {
    const MAX_INTERRUPT_NUMBER: usize = Self::MachineExternal as usize;

    #[inline]
    fn number(self) -> usize {
        self as usize
    }

    #[inline]
    fn from_number(value: usize) -> Result<Self, riscv::result::Error> {
        match value {
            // NC: 1 => Ok(Self::SupervisorSoft),
            3 => Ok(Self::MachineSoft),
            // NC: 5 => Ok(Self::SupervisorTimer),
            7 => Ok(Self::MachineTimer),
            // NC: 9 => Ok(Self::SupervisorExternal),
            11 => Ok(Self::MachineExternal),
            _ => Err(riscv::result::Error::InvalidVariant(value)),
        }
    }
}

/// SAFETY: `Interrupt` represents the standard RISC-V core interrupts
unsafe impl CoreInterruptNumber for CoreInterrupt {}
