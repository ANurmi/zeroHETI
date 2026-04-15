//! Test accesses to mailbox interface
#![no_main]
#![no_std]
mod common;

use fugit::ExtU64;

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC, apb_uart::ApbUart, asm_delay, i2c::I2c, interrupt::Interrupt,
    mmap::edfic::IE_BIT, mmio, mtimer::MTimer, nested_interrupt, rt::entry, sprintln,
};

use riscv::asm::wfi;

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

const fn parse_u32(s: &str) -> u32 {
    let mut out: u32 = 0;
    let mut i: usize = 0;
    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as u32;
        i += 1;
    }
    out
}

const LF: u32 = parse_u32(env!("LOAD_FACTOR"));
const PS: u32 = 10;
const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("zeroHETI control sim demonstrator");
    let mut i2c = I2c::init(4);

    init_intc();
    setup_irq(Interrupt::I2c);
    setup_irq(Interrupt::Mbx);
    setup_irq(Interrupt::MachineTimer);

    let mut mtimer = MTimer::instance().into_oneshot();
    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    sprintln!("Simulation configuration and parameters:");
    sprintln!(" - Simulation runtime (ms) : {}", SIM_PARAMS.hyperperiod_ms);
    sprintln!(" - Simulation prescaler    : {}", PS);
    sprintln!(" - Load factor     (0-100) : {}", LF);

    send_letter(SIM_PARAM_0_ADDR, 0xDEAD_BEEF);
    send_letter(SIM_PARAM_1_ADDR, 0xAAAA_AAAA);
    send_letter(SIM_PARAM_2_ADDR, 0xBBBB_BBBB);
    send_letter(SIM_PARAM_3_ADDR, 0xCCCC_CCCC);

    //i2c.read(0x68, &mut rbuf_4);

    unsafe { riscv::interrupt::enable() };
    i2c.irq_enable();

    // can't use global critical section if i2c driver requires
    i2c.write(0x60, &[0x67]);

    loop {
        wfi();
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
fn MachineTimer() {
    sprintln!("Mtimeirq");
    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();
}

#[nested_interrupt]
fn Mbx() {
    unsafe { riscv::interrupt::disable() };
    let addr = mmio::read_u32(MBX_IADD_ADDR as usize);
    let data = mmio::read_u32(MBX_IDAT_ADDR as usize);
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0100_0000);
    sprintln!("[ISR] read {:x} from {:x}", data, addr);
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0002_0000);
    unsafe { riscv::interrupt::enable() };
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
