pub const MBX_ADDR: usize = 0x10_8000;

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
    pub(crate) ibox: Letter,
    /// 0x14..0x1c Outbox
    pub(crate) obox: Letter,
}

bitflags::bitflags! {
    /// Status bit layout
    ///
    /// Based on obi_mbx.sv
    pub struct Stat: u32 {
        const IBOX_EMPT = 0b1 << 0;
        const IBOX_FULL = 0b1 << 1;
        const OBOX_EMPT = 0b1 << 2;
        const OBOX_FULL = 0b1 << 3;
    }

    /// Control bit layout
    ///
    /// Based on obi_mbx.sv
    pub struct Ctrl: u32 {
        /// Send outbox
        const OBOX_SEND = 0b1 << 0;
        /// Flush inbox
        const IBOX_FLSH = 0b1 << 8;
        /// Flush outbox
        const OBOX_FLSH = 0b1 << 9;
        /// Set IRQ
        const IRQ_SET = 0b1 << 16;
        /// Clear IRQ
        const IRQ_CLR = 0b1 << 17;
        /// OBI read acknowledge / pop inbox
        const READ_ACK = 0b1 << 24;
    }
}

/// Letter layout
///
/// Corresponds to inbox/outbox address layout.
#[repr(C)]
pub(crate) struct Letter {
    pub addr: u32,
    pub data: u32,
}
