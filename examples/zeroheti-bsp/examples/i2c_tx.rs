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
    mmio,
    rt::entry,
    sprintln,
};

use crate::common::{UART_BAUD, init_intc, pend_irq, setup_irq};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    // send letter
    //mmio::write_u32(MBX_CTRL_ADDR, 0x1);

    sprintln!("zeroHETI i2c test");
    let mut i2c = I2c::init(4);

    init_intc();

    setup_irq(ExternalInterrupt::I2c);

    let wbuf = [0; 4];
    let mut rbuf = [0; 4];

    unsafe { riscv::interrupt::enable() };

    i2c.i2c.irq_enable();
    i2c.write(0x68, &wbuf);
    i2c.irq_disable();

    i2c.write(0x60, &mut rbuf);

    //unsafe { I2c::instance() }.write(0x0, &wbuf);
    //unsafe { I2c::instance() }.read(0x0, &mut rbuf);

    //let m2_speed = u32::from_le_bytes(rbuf);

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[zeroheti_bsp::external_interrupt(ExternalInterrupt::I2c)]
fn i2c_isr() {
    //
    sprintln!("knob")
}

#[unsafe(export_name = "DefaultHandler")]
fn default_handler() {
    sprintln!("Hit default handler (unmapped interrupt)!");
    zeroheti_bsp::tb::rtl_tb_signal_fail();
}
