use crate::{
    mmap::i2c::*,
    mmio::{mask_u8, read_u8, unmask_u8, write_u32},
};

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

/// Write enable for I2C transaction, 0 for read and 1 for write
#[repr(u8)]
enum We {
    Read = 0,
    Write = 1,
}

enum Last {
    NotLast = 0,
    Last = 1,
}

impl<const BASE_ADDR: usize> I2cHal<BASE_ADDR> {
    /// # Parameters
    ///
    /// * `ps` - prescaler value, used to set the I2C clock frequency.
    #[inline]
    pub fn init(ps: u32) -> Self {
        let mut instance = Self;
        instance.set_prescaler(ps);
        instance.core_enable();
        instance
    }

    pub fn disable(mut self) {
        self.core_disable();
    }

    fn get_tip(&self) -> u8 {
        unsafe { read_u8(BASE_ADDR + I2C_STATUS_OFS) & (1 << 1) }
    }

    fn set_cmd(&mut self, cmd: Cmd) {
        write_u32(BASE_ADDR + I2C_CMD_OFS, cmd.bits() as u32);
    }

    fn send_addr_frame(&mut self, addr: u8, we: u8) {
        let tx_addr: u8 = addr << 1 | we;
        write_u32(BASE_ADDR + I2C_TX_OFS, tx_addr as u32);
        self.set_cmd(Cmd::STA | Cmd::WR);
        while self.get_tip() != 0 {}
    }

    fn send_data_frame(&mut self, data: u8, last: u8) {
        write_u32(BASE_ADDR + I2C_TX_OFS, data as u32);
        let mut cmd = Cmd::WR;
        if last == 1 {
            cmd |= Cmd::STO;
        }
        self.set_cmd(cmd);
        while self.get_tip() != 0 {}
    }

    fn recv_data_frame(&mut self, last: u8) -> u8 {
        let mut cmd = Cmd::RD;
        if last == 1 {
            cmd |= Cmd::STO;
        }
        self.set_cmd(cmd);
        while self.get_tip() != 0 {}
        unsafe { read_u8(BASE_ADDR + I2C_RX_OFS) }
    }

    pub fn set_prescaler(&mut self, val: u32) {
        write_u32(BASE_ADDR + I2C_CLK_PRESCALER_OFS, val);
    }

    fn core_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }

    fn core_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }

    pub fn irq_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }

    pub fn irq_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }

    pub fn write_tx(&mut self, addr: u8, buf: &[u8]) {
        self.send_addr_frame(addr, We::Write as u8);

        for byte in &buf[0..buf.len() - 1] {
            self.send_data_frame(*byte, Last::NotLast as u8);
        }
        self.send_data_frame(buf[buf.len() - 1], Last::Last as u8);
    }

    /// # Parameters
    ///
    /// * `buf` - buffer to store the read data, should have enough space for `read_len` bytes.
    ///           Existing bytes in the buffer will be overwritten up to
    ///           read_len.
    ///
    /// # Safety
    ///
    /// `buf` must have enough space for read_len bytes, and the caller must ensure that the I2C
    /// transaction is valid (e.g. the slave device at `addr` should be able to provide `read_len`
    /// bytes of data).
    pub fn read_tx(&mut self, addr: u8, buf: &mut [u8]) {
        self.send_addr_frame(addr, We::Read as u8);

        for idx in 0..(buf.len() - 1) {
            unsafe { *buf.get_unchecked_mut(idx) = self.recv_data_frame(Last::NotLast as u8) };
        }
        buf[buf.len() - 1] = self.recv_data_frame(Last::Last as u8);
    }
}
