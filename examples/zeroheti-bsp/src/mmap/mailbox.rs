pub const MBX_ADDR: usize = 0x3_0000;

pub const INBOX_OFS: usize = 0x00;
/// HW raises mailbox IRQ, when there is mail
pub const IRQ_ACK_OFS: usize = 0x04;
pub const TIME_LO_OFS: usize = 0x08;
pub const TIME_HI_OFS: usize = 0x0C;
pub const STAT_M0_OFS: usize = 0x10;
/*
pub const STAT_M1_OFS: usize = 0x14;
pub const STAT_M2_OFS: usize = 0x18;
pub const STAT_M3_OFS: usize = 0x1C;
*/
