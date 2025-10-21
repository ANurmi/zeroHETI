//! Print hello over UART
#![no_main]
#![no_std]

use riscv::{InterruptNumber, asm::nop, interrupt::Interrupt};
use zeroheti_bsp::{CPU_FREQ_HZ, apb_uart::ApbUart, asm_delay, hetic::Hetic, rt::entry, sprintln};

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    Hetic::line(22).set_level(6);
    Hetic::line(22).pend();

    Hetic::line(17).set_level(1);
    Hetic::line(17).pend();

    Hetic::line(3).set_level(8);
    Hetic::line(3).pend();

    Hetic::line(3).enable();
    Hetic::line(22).enable();
    Hetic::line(17).enable();

    unsafe { riscv::interrupt::enable() };
    for _ in 0..100 {
        nop();
    }

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(1_000_000);
        sprintln!("[UART] tick\r\n");
    }
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn custom_interrupt_handler() {
    sprintln!(
        "IRQ: {:#x?} = {:?}",
        riscv::register::mcause::read().code() & 0xfff,
        Interrupt::from_number(riscv::register::mcause::read().code() & 0xfff),
    );
}
