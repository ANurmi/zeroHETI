pub const MBX_ADDR: usize = 0x3_0000;

/// Register block for mailbox
#[repr(C)]
pub struct RegisterBlock {
    /// 0x0..0x4 Status
    pub stat: Stat,
    /// 0x4..0x8 Control
    pub ctrl: Ctrl,
    // 4 bytes of padding
    _pad: u32,
    /// 0xc..0x14 Inbox
    pub ibox: IoBox,
    /// 0x14..0x1c Outbox
    pub obox: IoBox,
}

bitflags::bitflags! {
    pub struct Stat: u32 {
        const OBOX_FULL = 0b1 << 2;
    }

    pub struct Ctrl: u32 {
        const DISPATCH = 0b1 << 0;
        const IRQ_CLEAR = 0b1 << 17;
        const POP_INBOX = 0b1 << 24;
    }
}

/// Inbox/outbox address layout
#[repr(C)]
pub struct IoBox {
    pub addr: u32,
    pub data: u32,
}
