//! Print hello over UART
#![no_main]
#![no_std]

use zeroheti_bsp::{CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, rt::entry, sprintln};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    serial.write_str("[UART] Hello from mock UART (Rust)!\r\n");
    serial.write_str("[UART] UART_TEST [PASSED]\r\n");

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}
