//! Test accesses to mailbox interface
#![no_main]
#![no_std]

use fugit::{ExtU32, ExtU64};

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::ApbUart,
    i2c::I2c,
    interrupt::Interrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mmio,
    mtimer::MTimer,
    nested_interrupt,
    riscv::{self, asm::nop, asm::wfi},
    rt::entry,
    sprintln,
    timer_group::{Periodic, Timer},
};

use riscv_rt::InterruptNumber;

// Mailbox addresses
const MBX_STAT_ADDR: u32 = 0x0003_0000;
const MBX_OBI_CTRL_ADDR: u32 = 0x0003_0004;
//const MBX_AXI_CTRL_ADDR: u32 = 0x0003_0008;
const _MBX_IADD_ADDR: u32 = 0x0003_000C;
const _MBX_IDAT_ADDR: u32 = 0x0003_0010;
const MBX_OADD_ADDR: u32 = 0x0003_0014;
const MBX_ODAT_ADDR: u32 = 0x0003_0018;

// Simulation configurations
const SIM_START_ADDR: u32 = 0x0100_0000;
const SIM_END_ADDR: u32 = 0x0100_0001;
const SIM_PRESCALER_ADDR: u32 = 0x0100_0002;
const SIM_LOADFACTOR_ADDR: u32 = 0x0100_0003;
const SIM_SEED_ADDR: u32 = 0x0100_0004;
//const SIM_TASK_PER_ADDR: u32 = 0x0100_0005;

const DL_MBX_ADDR: u32 = 0x0200_0000;
const DL_UPD_ADDR: u32 = 0x0200_0001;
const DL_CTRL_ADDR: u32 = 0x0200_0002;
const DL_REP_ADDR: u32 = 0x0200_0003;

const REP_TASK_PER_US: u32 = 700;
const TASK_MBX_ACK_ADDR: u32 = 0x0300_0000;

const TASK_CTRL_0_ADDR: u32 = 0x0301_0000;
const TASK_CTRL_1_ADDR: u32 = 0x0301_0001;
const TASK_CTRL_2_ADDR: u32 = 0x0301_0002;
const TASK_CTRL_3_ADDR: u32 = 0x0301_0003;

const TASK_REP_0_ADDR: u32 = 0x0401_0000;
const TASK_REP_1_ADDR: u32 = 0x0401_0001;
const TASK_REP_2_ADDR: u32 = 0x0401_0002;
const TASK_REP_3_ADDR: u32 = 0x0401_0003;

const TASK_UPD_0_ADDR: u32 = 0x0501_0000;
const TASK_UPD_1_ADDR: u32 = 0x0501_0001;
const TASK_UPD_2_ADDR: u32 = 0x0501_0002;
const TASK_UPD_3_ADDR: u32 = 0x0501_0003;

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
/*
 *const DL_WRN: u32 = 0x100;
 *const DL_REP: u32 = 0x200;
 */
struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("\n\r### Starting rt_prof (bare-metal) benchmark ###\n\r");
    let mut i2c = I2c::init(4);

    init_intc();
    setup_irq(Interrupt::I2c, 3);
    setup_irq(Interrupt::Mbx, 3);
    setup_irq(Interrupt::MachineTimer, u8::MAX);
    setup_irq(Interrupt::MachineExternal, u8::MAX);

    setup_irq(Interrupt::Timer0Ovf, 1);
    setup_irq(Interrupt::Timer1Ovf, 1);
    setup_irq(Interrupt::Timer2Ovf, 1);
    setup_irq(Interrupt::Timer3Ovf, 1);

    setup_irq(Interrupt::Timer0Cmp, 2);
    setup_irq(Interrupt::Timer1Cmp, 2);
    setup_irq(Interrupt::Timer2Cmp, 2);
    setup_irq(Interrupt::Timer3Cmp, 2);

    setup_irq(Interrupt::Ext0, 1);
    setup_irq(Interrupt::Ext1, 1);
    setup_irq(Interrupt::Ext2, 1);
    setup_irq(Interrupt::Ext3, 1);

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

    sprintln!("Configuration and parameters:");
    sprintln!(" - Runtime (ms)          : {}", SIM_PARAMS.hyperperiod_ms);
    sprintln!(" - Sim env prescaler     : {}", PS);
    sprintln!(" - Randomization seed    : 0x{:X}", SEED);
    sprintln!(" - Load factor  [0-100]  : {}", LF);
    sprintln!(" - Mailbox task DL  (us) : {}", DL_MBX);
    //sprintln!(" - Warning task DL  (us) : {}", DL_WRN);
    //sprintln!(" - Report  task DL  (us) : {}", DL_REP);
    sprintln!("");
    //sprintln!(" - Report  task per (us) : {}", REP_TASK_PER_US);

    send_letter(DL_MBX_ADDR, DL_MBX);
    send_letter(DL_UPD_ADDR, DL_MBX);
    send_letter(DL_CTRL_ADDR, DL_MBX);
    send_letter(DL_REP_ADDR, DL_MBX);
    /*
        send_letter(DL_WRN_ADDR, DL_WRN);
        send_letter(DL_REP_ADDR, DL_REP);
    */
    wait_outbox_empty();
    /*
        send_letter(SIM_SEED_ADDR, SEED);
        send_letter(SIM_LOADFACTOR_ADDR, LF);
    */
    send_letter(SIM_PRESCALER_ADDR, PS);
    wait_outbox_empty();

    //  send_letter(SIM_TASK_PER_ADDR, REP_TASK_PER_US);
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
    while (mmio::read_u32(MBX_STAT_ADDR as usize) & (1 << 2)) == 0 {}
}

#[inline]
fn send_letter(addr: u32, data: u32) {
    mmio::write_u32(MBX_OADD_ADDR as usize, addr);
    mmio::write_u32(MBX_ODAT_ADDR as usize, data);
    // send letter
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x1);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn MachineTimer() {
    riscv::interrupt::disable();

    send_letter(SIM_END_ADDR, 0x1);

    // Delay to let outbox clear
    for _ in 1..101 {
        nop();
    }

    let instret = riscv::register::minstret::read64();
    let active_time_cc = riscv::register::mcycle::read64();

    sprintln!("Instructions retired: {instret}, cycles: {active_time_cc}");

    // TODO: make neater
    let total_time_cc = RUNTIME_MS * 1000 * 100;

    sprintln!(
        "Total time (cc): {}, active time (cc): {},",
        total_time_cc,
        active_time_cc
    );
    sprintln!(
        "CPU utilization: {}%",
        (active_time_cc * 100) / total_time_cc
    );

    #[cfg(feature = "rtl-tb")]
    bsp::tb::rtl_tb_signal_ok();
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer0Cmp() {
    // TODO: less ham-fisted locking
    unsafe { riscv::interrupt::disable() };
    send_letter(TASK_CTRL_0_ADDR, 0x0);
    pend_irq(Interrupt::Ext0);
    send_letter(TASK_REP_0_ADDR, 0x1);
    unsafe { riscv::interrupt::enable() };
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer1Cmp() {
    unsafe { riscv::interrupt::disable() };
    send_letter(TASK_CTRL_1_ADDR, 0x0);
    pend_irq(Interrupt::Ext1);
    send_letter(TASK_REP_1_ADDR, 0x1);
    unsafe { riscv::interrupt::enable() };
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer2Cmp() {
    unsafe { riscv::interrupt::disable() };
    send_letter(TASK_CTRL_2_ADDR, 0x0);
    pend_irq(Interrupt::Ext2);
    send_letter(TASK_REP_2_ADDR, 0x1);
    unsafe { riscv::interrupt::enable() };
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer3Cmp() {
    unsafe { riscv::interrupt::disable() };
    send_letter(TASK_CTRL_3_ADDR, 0x0);
    pend_irq(Interrupt::Ext3);
    send_letter(TASK_REP_3_ADDR, 0x1);
    unsafe { riscv::interrupt::enable() };
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer0Ovf() {
    send_letter(TASK_UPD_0_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer1Ovf() {
    send_letter(TASK_UPD_1_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer2Ovf() {
    send_letter(TASK_UPD_2_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Timer3Ovf() {
    send_letter(TASK_UPD_3_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Ext0() {
    send_letter(TASK_REP_0_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Ext1() {
    send_letter(TASK_REP_1_ADDR, 0x0);
}
#[nested_interrupt]
#[allow(non_snake_case)]
fn Ext2() {
    //sprintln!("e2");
    send_letter(TASK_REP_2_ADDR, 0x0);
}
#[nested_interrupt]
#[allow(non_snake_case)]
fn Ext3() {
    //sprintln!("e3");
    send_letter(TASK_REP_3_ADDR, 0x0);
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn MachineExternal() {
    sprintln!("error!");
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn Mbx() {
    riscv::interrupt::disable();
    // Read inbox
    /*
    let addr = mmio::read_u32(MBX_IADD_ADDR as usize);
    let data = mmio::read_u32(MBX_IDAT_ADDR as usize);
    */
    // Inbox read ack
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0100_0000);

    // Ack MBX task
    send_letter(TASK_MBX_ACK_ADDR, 0x0);

    // IRQ clear
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0002_0000);

    send_letter(TASK_UPD_0_ADDR, 0x1);
    send_letter(TASK_UPD_1_ADDR, 0x1);
    send_letter(TASK_UPD_2_ADDR, 0x1);
    send_letter(TASK_UPD_3_ADDR, 0x1);

    pend_irq(Interrupt::Timer0Ovf);
    pend_irq(Interrupt::Timer1Ovf);
    pend_irq(Interrupt::Timer2Ovf);
    pend_irq(Interrupt::Timer3Ovf);

    unsafe { riscv::interrupt::enable() };
}

#[nested_interrupt]
#[allow(non_snake_case)]
fn I2c() {
    unsafe { I2c::instance() }.irq_ack();
}

#[unsafe(export_name = "DefaultHandler")]
fn default_handler() {
    sprintln!("Hit default handler (unmapped interrupt)!");
    bsp::tb::rtl_tb_signal_fail();
}

pub fn init_intc() {
    // HACK: clear mintstatus, required for zeroHETI
    unsafe { bsp::register::mintstatus::write(0.into()) };

    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::Clic;
        // Set level bits to 8
        Clic::smclicconfig().set_mnlbits(8);
    }
}

/// Setup `irq` for use with some basic defaults
///
/// Copy and customize this function if you need more involved configurations.
pub fn setup_irq(irq: impl InterruptNumber, _level: u8) {
    log::debug!("Set up IRQ (id = {})", irq.number());
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::{Clic, Polarity, Trig};

        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(_level);
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::{Edfic, Pol, Trig};

        Edfic::line(irq.number()).set_pol(Pol::Pos);
        Edfic::line(irq.number()).set_trig(Trig::Edge);
        Edfic::line(irq.number()).enable();
        Edfic::line(irq.number()).set_dl(0xffff_ffff);
    }
}

pub fn pend_irq(irq: impl InterruptNumber) {
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::CLIC;
        unsafe { CLIC::ip(irq).pend() }
    }
    #[cfg(feature = "intc-edfic")]
    {
        use zeroheti_bsp::edfic::Edfic;
        Edfic::line(irq.number()).pend();
    }
}
