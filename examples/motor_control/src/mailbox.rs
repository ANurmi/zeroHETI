//! Application-specific HAL for the mailbox
use bsp::{
    mmap::mailbox::*,
    mmio::{read_u32, write_u32},
};

#[repr(usize)]
pub enum MotorIdx {
    M0 = 0,
    M1 = 1,
    M2 = 2,
    M3 = 3,
}

pub type Mailbox = MailboxHal<MBX_ADDR>;

pub struct MailboxHal<const BASE_ADDR: usize> {
    inbox: InboxHal<BASE_ADDR>,
    outbox: OutboxHal<BASE_ADDR>,
}

impl<const BASE_ADDR: usize> MailboxHal<BASE_ADDR> {
    /// Retrieve an instance of the mailbox, comprising both halves: inbox and
    /// outbox. Call `split` to obtain the functional halves.
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self {
            inbox: InboxHal::<BASE_ADDR>::instance(),
            outbox: OutboxHal::<BASE_ADDR>::instance(),
        }
    }

    #[inline]
    pub fn split(self) -> (InboxHal<BASE_ADDR>, OutboxHal<BASE_ADDR>) {
        (self.inbox, self.outbox)
    }

    #[inline]
    pub fn merge(
        inbox: InboxHal<BASE_ADDR>,
        outbox: OutboxHal<BASE_ADDR>,
    ) -> MailboxHal<BASE_ADDR> {
        Self { inbox, outbox }
    }
}

pub type Inbox = InboxHal<MBX_ADDR>;
pub struct InboxHal<const BASE_ADDR: usize>;
impl<const BASE_ADDR: usize> InboxHal<BASE_ADDR> {
    /// Use [MailboxHal::instance]
    #[inline]
    const fn instance() -> Self {
        Self {}
    }

    /// Retrieve motor control instructions from the mailbox
    #[inline]
    pub fn read_inbox(&self) -> u32 {
        read_u32(BASE_ADDR + STAT_OFS)
    }

    /// Acknowledge that mailbox IRQ has been handled
    #[inline]
    pub fn ack_irq(&mut self) {
        write_u32(BASE_ADDR + CTRL_OFS, 1)
    }
}

pub type Outbox = OutboxHal<MBX_ADDR>;
pub struct OutboxHal<const BASE_ADDR: usize>;
impl<const BASE_ADDR: usize> OutboxHal<BASE_ADDR> {
    const TIME_LO_OFS: usize = 0x08;
    const TIME_HI_OFS: usize = 0x0C;
    const STAT_M0_OFS: usize = 0x10;

    /// Use [MailboxHal::instance]
    #[inline]
    const fn instance() -> Self {
        Self {}
    }

    /// Write time stamp, then a single motor status
    pub fn write_time_and_stat(&mut self, time: u64, stat: u32, motor: MotorIdx) {
        // Write time
        let time_lo = (time & 0xFFFF_FFFF) as u32;
        let time_hi = (time >> 32) as u32;
        write_u32(BASE_ADDR + Self::TIME_LO_OFS, time_lo);
        write_u32(BASE_ADDR + Self::TIME_HI_OFS, time_hi);

        // Write stat
        write_u32(BASE_ADDR + Self::STAT_M0_OFS + 0x4 * motor as usize, stat);
    }
}
