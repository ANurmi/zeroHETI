//! Print hello over UART
#![no_main]
#![no_std]

use riscv::{asm::nop, interrupt::Interrupt};
use zeroheti_bsp::{
    self as bsp, CPU_FREQ_HZ, apb_uart::ApbUart, asm_delay, mmap::hetic::*, rt::entry, sprintln,
};

fn set_ip(idx: u8) {
    unsafe { bsp::mmio::mask_u8(HETIC_BASE + LINE_SIZE * idx as usize, 0b1u8 << IP_OFS) };
}
fn clear_ip(idx: u8) {
    bsp::mmio::unmask_u8(HETIC_BASE + LINE_SIZE * idx as usize, 0b1u8 << IP_OFS);
}
fn set_ie(idx: u8) {
    bsp::mmio::mask_u8(HETIC_BASE + LINE_SIZE * idx as usize, 0b1u8 << IE_OFS);
}
fn clear_ie(idx: u8) {
    unsafe { bsp::mmio::unmask_u8(HETIC_BASE + LINE_SIZE * idx as usize, 0b1u8 << IE_OFS) };
}

fn set_prio(idx: u8, prio: u8) {
    unsafe { bsp::mmio::write_u8(HETIC_BASE + LINE_SIZE * idx as usize + 1, prio) };
}

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{}]", core::file!());

    set_prio(3, 8);
    set_ip(3);
    set_ie(3);

    unsafe { riscv::interrupt::enable() };
    for _ in 0..100 {
        nop();
    }

    #[cfg(feature = "rtl-tb")]
    bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(1_000_000);
        sprintln!("[UART] tick\r\n");
    }
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn custom_interrupt_handler() {
    sprintln!(
        "IRQ: {:#x?}",
        riscv::register::mcause::read().code() & 0xfff
    );
}
