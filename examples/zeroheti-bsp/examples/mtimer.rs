//! Print hello over UART
#![no_main]
#![no_std]
mod common;

use fugit::ExtU64;
use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, interrupt::CoreInterrupt,
    mtimer::MTimer, rt::entry, sprintln,
};

use crate::common::{init_intc, setup_irq};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    init_intc();
    setup_irq(CoreInterrupt::MachineTimer);

    let mut mtimer = MTimer::instance().into_oneshot();
    mtimer.start(10u64.micros());

    unsafe { riscv::interrupt::enable() };

    // Wait for 100 us
    asm_delay(NOPS_PER_SEC / 1_000 / 10);

    zeroheti_bsp::tb::signal_fail(Some(&mut serial));

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[zeroheti_bsp::core_interrupt(CoreInterrupt::MachineTimer)]
fn timeout() {
    zeroheti_bsp::tb::signal_pass(None);
}
