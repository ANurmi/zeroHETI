use core::ptr::{read_volatile, write_volatile};

use crate::mmap::mailbox::*;
use crate::mmio::{read_u32p, write_u32p};

/// Relocatable driver for mailbox
pub struct Mailbox(*mut RegisterBlock);
impl Mailbox {
    /// Retrieve an instance of the mailbox comprising both halves: inbox and
    /// outbox. Call `split` to obtain the functional halves.
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
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self(MBX_ADDR as *mut _)
    }

    #[inline]
    fn clear_irq(&mut self) {
        unsafe { write_volatile(&mut (*self.0).ctrl as *mut _, Ctrl::IRQ_CLEAR) };
    }

    /// Pop letter from inbox
    #[inline]
    fn pop_inbox(&mut self) {
        unsafe { write_volatile(&mut (*self.0).ctrl as *mut _, Ctrl::POP_INBOX) };
    }

    /// Returns `(addr, data)`
    #[inline]
    pub fn recv(&mut self) -> (u32, u32) {
        self.clear_irq();

        // Read inbox
        let addr = read_u32p(unsafe { &mut (*self.0).ibox.addr as *mut u32 });
        let data = read_u32p(unsafe { &mut (*self.0).ibox.data as *mut u32 });

        self.pop_inbox();

        (addr, data)
    }

    /// Returns `[(addr, data)]`
    #[inline]
    pub fn recv_many(&mut self, buf: &mut [(u32, u32)]) {
        self.clear_irq();

        for (addr, data) in buf.iter_mut() {
            // Read inbox
            *addr = read_u32p(unsafe { &mut (*self.0).ibox.addr as *mut u32 });
            *data = read_u32p(unsafe { &mut (*self.0).ibox.data as *mut u32 });
            self.pop_inbox();
        }
    }
}

pub struct Outbox(*mut RegisterBlock);
impl Outbox {
    #[inline]
    pub const unsafe fn instance() -> Self {
        Self(MBX_ADDR as *mut _)
    }

    #[inline]
    fn wait_outbox_empty(&self) {
        while (unsafe { read_volatile::<Stat>(&(*self.0).stat as *const _) })
            .intersection(Stat::OBOX_FULL)
            .is_empty()
        {}
    }

    #[inline]
    pub fn send(&mut self, addr: u32, data: u32) {
        // Ensure outbox has free capacity before sending
        self.wait_outbox_empty();

        // Write letter address and data
        write_u32p(unsafe { &mut (*self.0).obox.addr as *mut u32 }, addr);
        write_u32p(unsafe { &mut (*self.0).obox.data as *mut u32 }, data);

        // Dispatch
        unsafe { write_volatile(&mut (*self.0).ctrl as *mut _, Ctrl::DISPATCH) };
    }

    /// Sends multiple letters in a row. Hardware causes a wait in-between
    /// letters.
    ///
    /// Parameter is (addr, data)
    #[inline]
    pub fn send_many(&mut self, letters: &[(u32, u32)]) {
        // Ensure mailbox has free capacity before sending
        self.wait_outbox_empty();

        // Write all letters
        for (addr, data) in letters {
            write_u32p(unsafe { &mut (*self.0).obox.addr as *mut u32 }, *addr);
            write_u32p(unsafe { &mut (*self.0).obox.data as *mut u32 }, *data);

            // Dispatch
            unsafe { write_volatile(&mut (*self.0).ctrl as *mut _, Ctrl::DISPATCH) };
        }
    }
}
