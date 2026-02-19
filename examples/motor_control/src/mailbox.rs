use bsp::mmio::{read_u32, write_u32};

const MBX_ADDR: usize = 0x3_0000;

const INBOX_OFS: usize = 0x00;
/// HW raises mailbox IRQ, when there is mail
const IRQ_ACK_OFS: usize = 0x04;
const TIME_LO_OFS: usize = 0x08;
const TIME_HI_OFS: usize = 0x0C;
const M0_STAT_OFS: usize = 0x10;
/*
const M1_STAT_OFS: usize = 0x14;
const M2_STAT_OFS: usize = 0x18;
const M3_STAT_OFS: usize = 0x1C;
*/

#[repr(usize)]
pub enum Motor {
    M0 = 0,
    M1 = 1,
    M2 = 2,
    M3 = 3,
}

pub struct MailboxHal<const BASE_ADDR: usize>;

pub type Mailbox = MailboxHal<MBX_ADDR>;

impl<const BASE_ADDR: usize> MailboxHal<BASE_ADDR> {
    /// Retrieve motor control instructions from the mailbox
    pub fn read_inbox(&self) -> u32 {
        read_u32(BASE_ADDR + INBOX_OFS)
    }

    /// Acknowledge that mailbox IRQ has been handled
    pub fn ack_irq(&mut self) {
        write_u32(BASE_ADDR + IRQ_ACK_OFS, 1)
    }

    /// Write time stamp, then a single motor status
    pub fn write_time_and_stat(&mut self, time: u64, stat: u32, motor: Motor) {
        // Write time
        let time_lo = (time & 0xFFFF_FFFF) as u32;
        let time_hi = (time >> 32) as u32;
        write_u32(BASE_ADDR + TIME_LO_OFS, time_lo);
        write_u32(BASE_ADDR + TIME_HI_OFS, time_hi);

        // Write stat
        write_u32(BASE_ADDR + M0_STAT_OFS + 0x4 * motor as usize, stat);
    }
}
