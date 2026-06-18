/// Inbox/outbox address layout
pub struct IoBox {
    pub addr: u32,
    pub data: u32,
}

/// Mailbox address layout
pub struct AddrMbx {
    pub stat: u32,
    pub ctrl: u32,
    /// Inbox
    pub ib: IoBox,
    /// Outbox
    pub ob: IoBox,
}

pub const MAILBOX: AddrMbx = AddrMbx {
    stat: 0x0003_0000,
    ctrl: 0x0003_0004,
    ib: IoBox {
        addr: 0x0003_000C,
        data: 0x0003_0010,
    },
    ob: IoBox {
        addr: 0x0003_0014,
        data: 0x0003_0018,
    },
};
