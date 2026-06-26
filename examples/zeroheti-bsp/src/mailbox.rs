use core::ptr::{read_volatile, write_volatile};

use crate::mmap::mailbox::*;

/// Relocatable HAL for mailbox
pub struct Mailbox(*mut RegisterBlock);
impl Mailbox {
    /// Retrieve an instance of the mailbox comprising both halves: inbox and
    /// outbox. Call `split` on the instance to obtain the separate functional
    /// halves.
    ///
    /// # Safety
    ///
    /// This is the global instance. Ensure safe sharing.
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self(MBX_ADDR as *mut _)
    }

    #[inline]
    pub fn split(self) -> (Inbox, Outbox) {
        (Inbox(self.0), Outbox(self.0))
    }
}

pub struct Inbox(*mut RegisterBlock);
impl Inbox {
    /// # Safety
    ///
    /// This is the global instance. Ensure safe sharing.
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self(MBX_ADDR as *mut _)
    }

    #[inline]
    fn clear_irq(&mut self) {
        unsafe { write_volatile(&raw mut (*self.0).ctrl, Ctrl::IRQ_CLR) };
    }

    /// Pop letter from inbox
    #[inline]
    fn pop_inbox(&mut self) {
        unsafe { write_volatile(&raw mut (*self.0).ctrl, Ctrl::READ_ACK) };
    }

    /// Returns `(addr, data)`
    #[inline]
    pub fn recv(&mut self) -> (u32, u32) {
        // Read inbox
        let addr = unsafe { read_volatile(&raw mut (*self.0).ibox.addr) };
        let data = unsafe { read_volatile(&raw mut (*self.0).ibox.data) };

        self.pop_inbox();
        self.clear_irq();

        (addr, data)
    }

    /// Returns `[(addr, data)]`
    #[inline]
    pub fn recv_many(&mut self, buf: &mut [(u32, u32)]) {
        for (addr, data) in buf.iter_mut() {
            // Read inbox
            *addr = unsafe { read_volatile(&raw mut (*self.0).ibox.addr) };
            *data = unsafe { read_volatile(&raw mut (*self.0).ibox.data) };
            self.pop_inbox();
        }
        self.clear_irq();
    }
}

pub struct Outbox(*mut RegisterBlock);
impl Outbox {
    /// # Safety
    ///
    /// This is the global instance. Ensure safe sharing.
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self(MBX_ADDR as *mut _)
    }

    #[inline]
    fn wait_until_obox_empty(&self) {
        // Perform volatile read on the register until `OBOX_EMPT` becomes set
        while !unsafe { read_volatile::<Stat>(&raw const (*self.0).stat).contains(Stat::OBOX_EMPT) }
        {
        }
    }

    #[inline]
    pub fn send(&mut self, addr: u32, data: u32) {
        // Ensure outbox has free capacity before sending
        self.wait_until_obox_empty();

        unsafe {
            // Write letter address and data
            write_volatile(&raw mut (*self.0).obox.addr, addr);
            write_volatile(&raw mut (*self.0).obox.data, data);

            // Dispatch
            write_volatile(&raw mut (*self.0).ctrl, Ctrl::OBOX_SEND);
        }
    }
}
