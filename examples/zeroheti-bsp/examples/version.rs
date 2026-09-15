#![no_main]
#![no_std]

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, cfg_regs::CfgRegs, mmio, rt::entry, sprintln,
};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));
    let cfg_regs = CfgRegs::init();

    let hw_commit = cfg_regs.commit();
    let intc = if cfg_regs.intc_edfic() {"edfic"} else {"clic"};
    let uart = if cfg_regs.full_uart() {"full"} else {"mock"};
    let imem_bytes = cfg_regs.imem_bytes();
    let dmem_bytes = cfg_regs.dmem_bytes();

    sprintln!("zeroHETI HW build from commit:{:8x}", hw_commit);
    sprintln!("- Interrupt controller       : {intc}");
    sprintln!("- UART peripheral            : {uart}");
    sprintln!("- Instruction memory (bytes) : {imem_bytes}");
    sprintln!("- Data memory (bytes)        : {dmem_bytes}");
 
    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}
