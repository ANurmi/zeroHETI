use riscv::CoreInterruptNumber;
use riscv_types::InterruptNumber;
use strum::FromRepr;

#[derive(Clone, Copy, PartialEq, FromRepr)]
#[repr(usize)]
#[cfg_attr(not(feature = "ufmt"), derive(Debug))]
pub enum Interrupt {
    // NC: SupervisorSoft = 1,
    MachineSoft = 3,
    // NC: SupervisorTimer = 5,
    MachineTimer = 7,
    // NC: SupervisorExternal = 9,
    MachineExternal = 11,
    /// Mailbox
    Mbx = 16,
    SpiEvent0 = 17,
    SpiEvent1 = 18,
    I2c0 = 19,
    I2c1 = 20,
    Uart = 24,
    // Non-maskable interrupt, carried over from standard Ibex
    Nmi = 31, // reserved
    /// Timer0 overflow
    Timer0Ovf = 32,
    /// Timer0 compare
    Timer0Cmp = 33,
    Timer1Ovf = 34,
    Timer1Cmp = 35,
    Timer2Ovf = 36,
    Timer2Cmp = 37,
    Timer3Ovf = 38,
    Timer3Cmp = 39,
    Timer4Ovf = 40,
    Timer4Cmp = 41,
    Timer5Ovf = 42,
    Timer5Cmp = 43,
    Timer6Ovf = 44,
    Timer6Cmp = 45,
    Timer7Ovf = 46,
    Timer7Cmp = 47,
    Timer8Ovf = 48,
    Timer8Cmp = 49,
    Timer9Ovf = 50,
    Timer9Cmp = 51,
    Timer10Ovf = 52,
    Timer10Cmp = 53,
    Timer11Ovf = 54,
    Timer11Cmp = 55,
    Timer12Ovf = 56,
    Timer12Cmp = 57,
    Timer13Ovf = 58,
    Timer13Cmp = 59,
    Timer14Ovf = 60,
    Timer14Cmp = 61,
    Timer15Ovf = 62,
    Timer15Cmp = 63,

    /// Generic external interrupt 0
    Ext0 = 64,
    /// Generic external interrupt 1
    Ext1 = 65,
    /// Generic external interrupt 2
    Ext2 = 66,
    /// Generic external interrupt 3
    Ext3 = 67,
}

unsafe impl InterruptNumber for Interrupt {
    const MAX_INTERRUPT_NUMBER: usize = Self::Nmi as usize;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> Result<Self, riscv_types::result::Error> {
        Self::from_repr(value).ok_or(riscv_types::result::Error::InvalidVariant(value))
    }
}

/// SAFETY: `Interrupt` represents the standard RISC-V core interrupts
unsafe impl CoreInterruptNumber for Interrupt {}
