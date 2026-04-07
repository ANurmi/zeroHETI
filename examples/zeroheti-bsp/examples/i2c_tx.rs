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

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("zeroHETI i2c test");
    let mut i2c = I2c::init(4);

    init_intc();

    setup_irq(ExternalInterrupt::I2c);

    // 0xDEADBEEF
    let wbuf_0 = [0xEF, 0xBE, 0xAD, 0xDE];
    let wbuf_1 = [0x55, 0x7A, 0xEA];
    let mut rbuf_4 = [0; 4];
    let mut rbuf_1 = [0];
    unsafe { riscv::interrupt::enable() };
    i2c.irq_enable();

    i2c.write(0x60, &wbuf_0);
    i2c.read(0x68, &mut rbuf_4);
    i2c.read(0x11, &mut rbuf_1);
    i2c.write(0x01, &wbuf_1);

    let rdata_4 = u32::from_le_bytes(rbuf_4);
    let rdata_1 = u8::from_le_bytes(rbuf_1);

    sprintln!("SW read 4: {:x}", rdata_4);
    sprintln!("SW read 1: {:x}", rdata_1);
    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
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
