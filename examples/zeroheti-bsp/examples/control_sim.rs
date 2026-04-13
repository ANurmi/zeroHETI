//! Test accesses to mailbox interface
#![no_main]
#![no_std]
mod common;

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC,
    apb_uart::ApbUart,
    asm_delay,
    i2c::I2c,
    interrupt::{CoreInterrupt, ExternalInterrupt},
    mmap::edfic::IE_BIT,
    mmio, nested_interrupt,
    rt::entry,
    sprintln,
};

use crate::common::{UART_BAUD, init_intc, pend_irq, setup_irq};

const MBX_STAT_ADDR: u32 = 0x0003_0000;
const MBX_OBI_CTRL_ADDR: u32 = 0x0003_0004;
//let MBX_AXI_CTRL_ADDR = 0x0003_0008;
const MBX_IADD_ADDR: u32 = 0x0003_000C;
const MBX_IDAT_ADDR: u32 = 0x0003_0010;
const MBX_OADD_ADDR: u32 = 0x0003_0014;
const MBX_ODAT_ADDR: u32 = 0x0003_0018;

const SIM_PARAM_0_ADDR: u32 = 0x0100_0000;
const SIM_PARAM_1_ADDR: u32 = 0x0200_0000;
const SIM_PARAM_2_ADDR: u32 = 0x0300_0000;
const SIM_PARAM_3_ADDR: u32 = 0x0400_0000;

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("zeroHETI control sim demonstrator");
    let mut i2c = I2c::init(4);
    init_intc();
    setup_irq(ExternalInterrupt::I2c);
    unsafe { riscv::interrupt::enable() };
    i2c.irq_enable();

    sprintln!("TODO: program sim env configuration");

    send_letter(SIM_PARAM_0_ADDR, 0xDEAD_BEEF);
    send_letter(SIM_PARAM_1_ADDR, 0xAAAA_AAAA);
    send_letter(SIM_PARAM_2_ADDR, 0xBBBB_BBBB);
    send_letter(SIM_PARAM_3_ADDR, 0xCCCC_CCCC);

    /*
        mmio::write_u32(MBX_OADD_ADDR, 0x0100_0000);
        mmio::write_u32(MBX_ODAT_ADDR, 0xDEAD_BEEF);
        // send letter
        mmio::write_u32(MBX_OBI_CTRL_ADDR, 0x1);

        mmio::write_u32(MBX_OADD_ADDR, 0x0200_0000);
        mmio::write_u32(MBX_ODAT_ADDR, 0xAAAA_AAAA);
        mmio::write_u32(MBX_OBI_CTRL_ADDR, 0x1);
        // send letter
    */
    i2c.write(0x60, &[0x67]);

    /*
        i2c.write(0x60, &wbuf_0);
        i2c.read(0x68, &mut rbuf_4);
        i2c.read(0x11, &mut rbuf_1);
        i2c.write(0x01, &wbuf_1);

        let rdata_4 = u32::from_le_bytes(rbuf_4);
        let rdata_1 = u8::from_le_bytes(rbuf_1);

        sprintln!("SW read 4: {:x}", rdata_4);
        sprintln!("SW read 1: {:x}", rdata_1);
    */
    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[inline]
fn send_letter(addr: u32, data: u32) {
    mmio::write_u32(MBX_OADD_ADDR as usize, addr);
    mmio::write_u32(MBX_ODAT_ADDR as usize, data);
    // send letter
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x1);
}

#[nested_interrupt]
fn Mbx() {
    sprintln!("Ding dong");
}

#[nested_interrupt]
fn I2c() {
    unsafe { I2c::instance() }.irq_ack();
}

#[unsafe(export_name = "DefaultHandler")]
fn default_handler() {
    sprintln!("Hit default handler (unmapped interrupt)!");
    zeroheti_bsp::tb::rtl_tb_signal_fail();
}
