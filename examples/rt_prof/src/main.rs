#![no_main]
#![no_std]

use fugit::{ExtU32, ExtU64};

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::ApbUart,
    core_interrupt,
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
const MBX_IADD_ADDR: u32 = 0x0003_000C;
const MBX_IDAT_ADDR: u32 = 0x0003_0010;
const MBX_OADD_ADDR: u32 = 0x0003_0014;
const MBX_ODAT_ADDR: u32 = 0x0003_0018;

// Simulation configurations
const SIM_START_ADDR: u32 = 0x0100_0000;
const SIM_END_ADDR: u32 = 0x0100_0001;
const SIM_PRESCALER_ADDR: u32 = 0x0100_0002;
const SIM_LOADFACTOR_ADDR: u32 = 0x0100_0003;
const SIM_SEED_ADDR: u32 = 0x0100_0004;

const DL_MBX_ADDR: u32 = 0x0200_0000;
const DL_UPD_ADDR: u32 = 0x0200_0001;
const DL_CTRL_ADDR: u32 = 0x0200_0002;
const DL_REP_ADDR: u32 = 0x0200_0003;

const I2C_M0_ADDR: u8 = 0x10;
const I2C_M1_ADDR: u8 = 0x11;
const I2C_M2_ADDR: u8 = 0x12;
const I2C_M3_ADDR: u8 = 0x13;

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

const DL_MBX: u32 = 1000;

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

// Infer task period from DL and LF
const MBX_PER: u32 = 3 * DL_MBX + (4 * (100 - LF));

const PRIO_CTRL: u8 = 4;
const PRIO_MAIL: u8 = 5;
const PRIO_UPD: u8 = 3;
const PRIO_REP: u8 = 1;

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("\n\r### Starting rt_prof (bare-metal) benchmark ###\n\r");
    let mut i2c = I2c::init(4);

    //assert!(MBX_PER > DL_MBX, "Mailbox task period too short!\n\r");

    init_intc();

    setup_irq(Interrupt::MachineTimer, u8::MAX);
    setup_irq(Interrupt::MachineExternal, u8::MAX);

    setup_irq(Interrupt::Mbx, PRIO_MAIL);

    setup_irq(Interrupt::Timer0Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer1Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer2Ovf, PRIO_UPD);
    setup_irq(Interrupt::Timer3Ovf, PRIO_UPD);

    setup_irq(Interrupt::Timer0Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer1Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer2Cmp, PRIO_CTRL);
    setup_irq(Interrupt::Timer3Cmp, PRIO_CTRL);

    setup_irq(Interrupt::Ext0, PRIO_REP);
    setup_irq(Interrupt::Ext1, PRIO_REP);
    setup_irq(Interrupt::Ext2, PRIO_REP);
    setup_irq(Interrupt::Ext3, PRIO_REP);

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
    sprintln!(" - Mailbox period   (us) : {}", MBX_PER);
    sprintln!("");

    send_letter(DL_MBX_ADDR, DL_MBX);
    send_letter(DL_UPD_ADDR, DL_MBX);
    send_letter(DL_CTRL_ADDR, DL_MBX);
    send_letter(DL_REP_ADDR, DL_MBX);
    wait_outbox_empty();

    send_letter(SIM_PRESCALER_ADDR, PS);
    send_letter(SIM_LOADFACTOR_ADDR, LF);
    wait_outbox_empty();

    send_letter(SIM_START_ADDR, 0x0);

    timers.iter_mut().for_each(Periodic::start);
    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    unsafe { riscv::interrupt::enable() };

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

#[inline]
fn pend_task(addr: u32) {
    send_letter(addr, 0x1);
}

#[inline]
fn ack_task(addr: u32) {
    send_letter(addr, 0x0);
}

#[core_interrupt(bsp::interrupt::Interrupt::MachineTimer)]
fn irq_mtime() {
    riscv::interrupt::disable();

    // Terminate scoreboard
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

#[core_interrupt(bsp::interrupt::Interrupt::Timer0Cmp)]
fn irq_timer_cmp0() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.read(I2C_M0_ADDR, &mut rbuf); // Read motor status
    unsafe { riscv::interrupt::enable() };

    let rdata = u8::from_le_bytes(rbuf);

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.write(I2C_M0_ADDR, &[0x10]); // Write to motor
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_CTRL_0_ADDR); // acknowledge this task
    pend_irq(Interrupt::Ext0); // Pend report task (HW)
    pend_task(TASK_REP_0_ADDR); // Pend report task (TB)

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Cmp)]
fn irq_timer_cmp1() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.read(I2C_M1_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let rdata = u8::from_le_bytes(rbuf);

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.write(I2C_M1_ADDR, &[0x11]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_CTRL_1_ADDR);
    pend_irq(Interrupt::Ext1);
    pend_task(TASK_REP_1_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Cmp)]
fn irq_timer_cmp2() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];
    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.read(I2C_M2_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let rdata = u8::from_le_bytes(rbuf);

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.write(I2C_M2_ADDR, &[0x12]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_CTRL_2_ADDR);
    pend_irq(Interrupt::Ext2);
    pend_task(TASK_REP_2_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Cmp)]
fn irq_timer_cmp3() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.read(I2C_M3_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let rdata = u8::from_le_bytes(rbuf);

    unsafe { riscv::interrupt::disable() };
    unsafe { I2c::instance() }.write(I2C_M3_ADDR, &[0x13]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_CTRL_3_ADDR);
    pend_irq(Interrupt::Ext3);
    pend_task(TASK_REP_3_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer0Ovf)]
fn irq_timer_ovf0() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_UPD_0_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Ovf)]
fn irq_timer_ovf1() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_UPD_1_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Ovf)]
fn irq_timer_ovf2() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_UPD_2_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Ovf)]
fn irq_timer_ovf3() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASK_UPD_3_ADDR);

    // disable nesting
    unsafe { riscv::interrupt::disable() };
    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext0)]
fn irq_ext0() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    //unsafe { riscv::interrupt::enable() };

    ack_task(TASK_REP_0_ADDR);

    //unsafe { riscv::interrupt::disable() };
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext1)]
fn irq_ext1() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    //unsafe { riscv::interrupt::enable() };

    ack_task(TASK_REP_1_ADDR);

    //unsafe { riscv::interrupt::disable() };
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext2)]
fn irq_ext2() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    //unsafe { riscv::interrupt::enable() };

    ack_task(TASK_REP_2_ADDR);

    //unsafe { riscv::interrupt::disable() };
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext3)]
fn irq_ext3() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    //unsafe { riscv::interrupt::enable() };

    ack_task(TASK_REP_3_ADDR);

    //unsafe { riscv::interrupt::disable() };
    bsp::register::mintthresh::write(last_mintthresh.into());
}

/* Not used
#[nested_interrupt]
#[allow(non_snake_case)]
fn MachineExternal() {
    sprintln!("error!");
}*/

#[core_interrupt(bsp::interrupt::Interrupt::Mbx)]
fn irq_mbx() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_MAIL as usize).into());
    unsafe { riscv::interrupt::enable() };

    // Read inbox
    /*
    let addr = mmio::read_u32(MBX_IADD_ADDR as usize);
    let data = mmio::read_u32(MBX_IDAT_ADDR as usize);
    */

    for i in 0..4 {
        let addr = mmio::read_u32(MBX_IADD_ADDR as usize);
        let data = mmio::read_u32(MBX_IDAT_ADDR as usize);
        //sprintln!("A: {:X}, D: {:X}", addr, data);
        // Inbox read ack
        mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0100_0000);
    }

    // IRQ clear
    mmio::write_u32(MBX_OBI_CTRL_ADDR as usize, 0x0002_0000);

    pend_task(TASK_UPD_0_ADDR);
    pend_task(TASK_UPD_1_ADDR);
    pend_task(TASK_UPD_2_ADDR);
    pend_task(TASK_UPD_3_ADDR);

    pend_irq(Interrupt::Timer0Ovf);
    pend_irq(Interrupt::Timer1Ovf);
    pend_irq(Interrupt::Timer2Ovf);
    pend_irq(Interrupt::Timer3Ovf);

    // Ack MBX task
    ack_task(TASK_MBX_ACK_ADDR);

    riscv::interrupt::disable();
    bsp::register::mintthresh::write(last_mintthresh.into());
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
