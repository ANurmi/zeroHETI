//! Print hello over UART
#![no_main]
#![no_std]

use zeroheti_bsp::{CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, mmio, rt::entry, sprintln};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    let CFG_BASE_ADDR = 0x0000_3500;

    let rd = mmio::read_u32(CFG_BASE_ADDR);

    let git_hash = "deadbeefbollocks";
    sprintln!("zeroHETI HW build from commit: {:8x}", rd);

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}
