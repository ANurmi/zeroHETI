//! Test accesses to mailbox interface
#![no_main]
#![no_std]

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, mmio, rt::entry, sprintln,
};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    const MBX_STAT_ADDR: usize = 0x0003_0000;
    const MBX_CTRL_ADDR: usize = 0x0003_0004;
    const _MBX_IADD_ADDR: usize = 0x0003_0008;
    const _MBX_IDAT_ADDR: usize = 0x0003_000C;
    const MBX_OADD_ADDR: usize = 0x0003_0010;
    const MBX_ODAT_ADDR: usize = 0x0003_0014;

    let letter_0 = 0x2b00b1e5;

    let rd = mmio::read_u32(MBX_STAT_ADDR);
    mmio::write_u32(MBX_OADD_ADDR, 0x1000);
    mmio::write_u32(MBX_ODAT_ADDR, letter_0);

    // send letter
    mmio::write_u32(MBX_CTRL_ADDR, 0x1);
    // flush outbox
    mmio::write_u32(MBX_CTRL_ADDR, 0x1 << 9);
    // irq set
    mmio::write_u32(MBX_CTRL_ADDR, 0x1 << 16);
    // irq clear
    mmio::write_u32(MBX_CTRL_ADDR, 0x1 << 17);

    sprintln!("zeroHETI mailbox test, rd: {:X}", rd);

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}
