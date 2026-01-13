use crate::hetic::InterruptNumber;
use riscv_pac::ExternalInterruptNumber;
use strum::FromRepr;

// Re-export core interrupts
pub use crate::riscv::interrupt::machine::Interrupt as CoreInterrupt;

#[derive(Clone, Copy, PartialEq, FromRepr)]
#[repr(usize)]
#[cfg_attr(not(feature = "ufmt"), derive(Debug))]
pub enum ExternalInterrupt {
    /// UART interrupt (Non-standard, overrides M-mode software interrupt
    /// mapping.)
    Uart = 3,
    // Non-standard - overrides something
    I2c = 11,
    //Gpio = 18,
    //SpiRxTxIrq = 19,
    /// SPI end of transmission
    //SpiEotIrq = 20,
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
    /// Timer queue interrupt on `~full` -> `full` transition.
    //TqFull = 29,
    /// Timer queue interrupt on `full` -> `~full` transition.
    //TqNotFull = 30,
    /// Non-maskable interrupt, carried over from standard Ibex
    Nmi = 31,
}

unsafe impl ExternalInterruptNumber for ExternalInterrupt {}

unsafe impl InterruptNumber for ExternalInterrupt {
    const MAX_INTERRUPT_NUMBER: usize = 255;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> Result<Self, riscv_pac::result::Error> {
        Self::from_repr(value).ok_or(riscv_pac::result::Error::InvalidVariant(value))
    }
}
