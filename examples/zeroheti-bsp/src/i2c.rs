use crate::{
    mmap::i2c::*,
    mmio::{mask_u8, read_u8, unmask_u8, write_u32},
};

pub struct I2c<const BASE_ADDR: usize>;

impl<const BASE_ADDR: usize> I2c<BASE_ADDR> {
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

    pub fn get_tip(&self) -> u8 {
        unsafe { read_u8(BASE_ADDR + I2C_STATUS_OFS) & (1 << 1) }
    }

    pub fn set_cmd(&mut self, sta: u8, sto: u8, we: u8, ack: u8, ia: u8) {
        let r: u8 = if we == 0 { 1 } else { 0 };
        //                 STA    STO      RD      WR     ACK      IA
        let cmd: u32 = (sta << 7 | sto << 6 | r << 5 | we << 4 | ack << 3 | ia << 0) as u32;
        write_u32(BASE_ADDR + I2C_CMD_OFS, cmd);
    }

    pub fn send_addr_frame(&mut self, addr: u8, we: u8) {
        let tx_addr: u8 = addr << 1 | we;
        write_u32(BASE_ADDR + I2C_TX_OFS, tx_addr as u32);
        self.set_cmd(1, 0, 1, 0, 0);
        while self.get_tip() != 0 {}
    }

    pub fn send_data_frame(&mut self, data: u8, last: u8) {
        write_u32(BASE_ADDR + I2C_TX_OFS, data as u32);
        self.set_cmd(0, last, 1, 0, 0);
        while self.get_tip() != 0 {}
    }

    pub fn recv_data_frame(&mut self, last: u8) -> u8 {
        self.set_cmd(0, last, 0, 0, 0);
        while self.get_tip() != 0 {}
        unsafe { read_u8(BASE_ADDR + I2C_RX_OFS) }
    }

    pub fn set_prescaler(&mut self, val: u32) {
        write_u32(BASE_ADDR + I2C_CLK_PRESCALER_OFS, val);
    }

    pub fn core_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }

    pub fn core_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 7);
    }

    pub fn irq_enable(&mut self) {
        mask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }

    pub fn irq_disable(&mut self) {
        unmask_u8(BASE_ADDR + I2C_CTRL_OFS, 1 << 6);
    }
}
