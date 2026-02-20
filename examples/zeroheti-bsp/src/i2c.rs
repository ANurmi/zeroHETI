use crate::{
    mmap::i2c::*,
    mmio::{mask_u8, read_u8, unmask_u8, write_u8, write_u16},
};
use embedded_hal::i2c::{Operation, SevenBitAddress};

pub struct I2cHal<const BASE_ADDR: usize>;

pub type I2c = I2cHal<I2C_BASE>;

bitflags::bitflags! {
    struct Cmd: u8 {
        const IA  = 0b0000_0001;
        const ACK = 0b0000_1000;
        /// Write
        const WR  = 0b0001_0000;
        /// Read
        const RD  = 0b0010_0000;
        /// Stop
        const STO = 0b0100_0000;
        /// Start
        const STA = 0b1000_0000;
    }
}

bitflags::bitflags! {
    /// Write enable for I2C transaction, 0 for read and 1 for write
    struct We: u8 {
        const READ  = 0b0;
        const WRITE = 0b1;
    }
}

bitflags::bitflags! {
    #[derive(PartialEq)]
    struct Last: u8 {
        const NOT_LAST = 0b0;
        const LAST     = 0b1;
    }
}

impl<const BASE_ADDR: usize> I2cHal<BASE_ADDR> {
    /// # Parameters
    ///
    /// * `ps` - prescaler value, used to set the I2C clock frequency.
    #[inline]
    pub fn init(ps: u16) -> Self {
        let mut instance = Self;
        instance.set_prescaler(ps);
        instance.core_enable();
        instance
    }

    #[inline]
    pub fn disable(mut self) {
        self.core_disable();
    }

    #[inline]
    pub fn set_prescaler(&mut self, val: u16) {
        // Safety: aligned on zeroHETI
        unsafe { write_u16(BASE_ADDR + I2C_CLK_PRESCALER_OFS, val) };
    }

    #[inline]
    pub fn irq_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }

    #[inline]
    pub fn irq_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }

    #[inline]
    fn get_tip(&self) -> u8 {
        // Safety: aligned on zeroHETI
        unsafe { read_u8(BASE_ADDR + I2C_STATUS_OFS) & (1 << 1) }
    }

    #[inline]
    fn set_cmd(&mut self, cmd: Cmd) {
        // Safety: aligned on zeroHETI
        unsafe { write_u8(BASE_ADDR + I2C_CMD_OFS, cmd.bits()) };
    }

    #[inline]
    fn send_addr_frame(&mut self, addr: u8, we: We) {
        let addr: u8 = addr << 1 | we.bits();
        // Safety: aligned on zeroHETI
        unsafe { write_u8(BASE_ADDR + I2C_TX_OFS, addr) };
        self.set_cmd(Cmd::STA | Cmd::WR);
        while self.get_tip() != 0 {}
    }

    fn send_data_frames(&mut self, buf: &[u8]) {
        for byte in &buf[0..buf.len() - 1] {
            // Safety: aligned on zeroHETI
            unsafe { write_u8(BASE_ADDR + I2C_TX_OFS, *byte) };
            self.set_cmd(Cmd::WR);
            while self.get_tip() != 0 {}
        }

        // Send last byte with stop condition
        // Safety: aligned on zeroHETI
        unsafe { write_u8(BASE_ADDR + I2C_TX_OFS, buf[buf.len() - 1]) };
        self.set_cmd(Cmd::WR | Cmd::STO);
        while self.get_tip() != 0 {}
    }

    fn recv_data_frames(&mut self, buf: &mut [u8]) {
        let last_idx = buf.len() - 1;
        for byte in &mut buf[0..last_idx] {
            self.set_cmd(Cmd::RD);
            while self.get_tip() != 0 {}
            *byte = unsafe { read_u8(BASE_ADDR + I2C_RX_OFS) };
        }

        // Read last byte with stop condition
        self.set_cmd(Cmd::RD | Cmd::STO);
        while self.get_tip() != 0 {}
        // Safety: aligned on zeroHETI
        buf[last_idx] = unsafe { read_u8(BASE_ADDR + I2C_RX_OFS) };
    }

    #[inline]
    fn core_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }

    #[inline]
    fn core_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    // ...
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match *self {
            // ...
        }
    }
}

impl<const BASE_ADDR: usize> embedded_hal::i2c::ErrorType for I2cHal<BASE_ADDR> {
    type Error = Error;
}

impl<const BASE_ADDR: usize> embedded_hal::i2c::I2c<SevenBitAddress> for I2cHal<BASE_ADDR> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for op in operations {
            match op {
                Operation::Read(buf) => {
                    self.read(address, buf)?;
                }
                Operation::Write(buf) => {
                    self.write(address, buf)?;
                }
            }
        }
        Ok(())
    }

    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.send_addr_frame(address, We::READ);
        self.recv_data_frames(read);
        Ok(())
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.send_addr_frame(address, We::WRITE);
        self.send_data_frames(write);
        Ok(())
    }
}
