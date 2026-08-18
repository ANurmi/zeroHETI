#![no_main]
#![no_std]

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, mmio, rt::entry, sprintln,
};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    const CFG_BASE_ADDR: usize = 0x0000_4000;

    let rd = mmio::read_u32(CFG_BASE_ADDR);
    let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);

    let intc_type = if (cfg & 0b1) == 0b1 { "EDFIC" } else { "CLIC" };
    let imem_bytes = 2u32.pow((cfg & 0x00FF00) >> 8);
    let dmem_bytes = 2u32.pow((cfg & 0xFF0000) >> 16);

    sprintln!("zeroHETI HW build from commit: {:8x}", rd);
    sprintln!("Platform config: {:08x}", cfg);
    sprintln!("- Interrupt controller       : {intc_type}");
    sprintln!("- Instruction memory (bytes) : {imem_bytes}");
    sprintln!("- Data memory (bytes)        : {dmem_bytes}");

    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}
