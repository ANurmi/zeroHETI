//! Test accesses to mailbox interface
#![no_main]
#![no_std]
mod common;

use fugit::{ExtU32, ExtU64};

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC,
    apb_uart::ApbUart,
    asm_delay,
    i2c::I2c,
    interrupt::Interrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mmap::edfic::IE_BIT,
    mmio,
    mtimer::MTimer,
    nested_interrupt,
    rt::entry,
    sprintln,
    timer_group::{Periodic, Timer},
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

const SIM_START_ADDR: u32 = 0x0100_0000;
const SIM_END_ADDR: u32 = 0x0100_0001;
const SIM_PRESCALER_ADDR: u32 = 0x0100_0002;
const SIM_LOADFACTOR_ADDR: u32 = 0x0100_0003;
const SIM_SEED_ADDR: u32 = 0x0100_0004;

const DL_MBX_ADDR: u32 = 0x0200_0000;
const DL_WRN_ADDR: u32 = 0x0200_0001;
const DL_REP_ADDR: u32 = 0x0200_0002;

const REP_TASK_PER_US: u32 = 500;

const TASK_MBX_ACK_ADDR: u32 = 0x0300_0000;
const TASK_REP_0_ADDR: u32 = 0x0301_0000;
const TASK_REP_1_ADDR: u32 = 0x0301_0001;
const TASK_REP_2_ADDR: u32 = 0x0301_0002;
const TASK_REP_3_ADDR: u32 = 0x0301_0003;

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
const SEED: u32 = 0xB0110c55;
const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

const DL_MBX: u32 = 0x200;
const DL_WRN: u32 = 0x80;
const DL_REP: u32 = 0x100;

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
    setup_irq(Interrupt::Timer0Cmp);
    setup_irq(Interrupt::Timer1Cmp);
    setup_irq(Interrupt::Timer2Cmp);
    setup_irq(Interrupt::Timer3Cmp);

    let timers = &mut [
        Timer::init::<TIMER0_ADDR>().into_periodic(),
        Timer::init::<TIMER1_ADDR>().into_periodic(),
        Timer::init::<TIMER2_ADDR>().into_periodic(),
        Timer::init::<TIMER3_ADDR>().into_periodic(),
    ];

    timers[0].set_period(REP_TASK_PER_US.micros());
    timers[1].set_period(REP_TASK_PER_US.micros());
    timers[2].set_period(REP_TASK_PER_US.micros());
    timers[3].set_period(REP_TASK_PER_US.micros());

    let mut mtimer = MTimer::instance().into_oneshot();

    sprintln!("Simulation configuration and parameters:");
    sprintln!(" - Runtime (ms)         : {}", SIM_PARAMS.hyperperiod_ms);
    sprintln!(" - Prescaler            : {}", PS);
    sprintln!(" - Seed                 : 0x{:X}", SEED);
    sprintln!(" - Load factor  [0-100] : {}", LF);
    sprintln!(" - Mailbox task DL (us) : {}", DL_MBX);
    sprintln!(" - Warning task DL (us) : {}", DL_WRN);
    sprintln!(" - Report  task DL (us) : {}", DL_REP);

    send_letter(DL_MBX_ADDR, DL_MBX);
    send_letter(DL_WRN_ADDR, DL_WRN);
    send_letter(DL_REP_ADDR, DL_REP);

    wait_outbox_empty();

    send_letter(SIM_SEED_ADDR, SEED);
    send_letter(SIM_LOADFACTOR_ADDR, LF);
    send_letter(SIM_PRESCALER_ADDR, PS);
    send_letter(SIM_START_ADDR, 0x0);
    //i2c.read(0x68, &mut rbuf_4);

    timers.iter_mut().for_each(Periodic::start);

    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    unsafe { riscv::interrupt::enable() };
    i2c.irq_enable();

    // can't use global critical section if i2c driver requires
    i2c.write(0x60, &[0x67]);

    loop {
        wfi();
    }
}

#[inline]
fn wait_outbox_empty() {
    while ((mmio::read_u32(MBX_STAT_ADDR as usize) & (1 << 2)) == 0) {}
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
    unsafe { riscv::interrupt::disable() };
    send_letter(SIM_END_ADDR, 0x0);
    #[cfg(feature = "rtl-tb")]
    zeroheti_bsp::tb::rtl_tb_signal_ok();
}

#[nested_interrupt]
fn Timer0Cmp() {
    sprintln!("y0");
    send_letter(TASK_REP_0_ADDR, 0x0);
}

#[nested_interrupt]
fn Timer1Cmp() {
    sprintln!("y1");
    send_letter(TASK_REP_1_ADDR, 0x0);
}

#[nested_interrupt]
fn Timer2Cmp() {
    sprintln!("y2");
    send_letter(TASK_REP_2_ADDR, 0x0);
}

#[nested_interrupt]
fn Timer3Cmp() {
    sprintln!("y3");
    send_letter(TASK_REP_3_ADDR, 0x0);
}

#[nested_interrupt]
fn Mbx() {
    unsafe { riscv::interrupt::disable() };
    // Read inbox
    let addr = mmio::read_u32(MBX_IADD_ADDR as usize);
    let data = mmio::read_u32(MBX_IDAT_ADDR as usize);
    // Inbox read ack
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0100_0000);
    sprintln!("[MBX]");

    // Ack MBX task
    send_letter(TASK_MBX_ACK_ADDR, 0x0);

    // IRQ clear
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
