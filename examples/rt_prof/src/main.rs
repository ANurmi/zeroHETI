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
    riscv::{self, asm::nop, asm::wfi},
    rt::entry,
    sprintln,
    timer_group::{Periodic, Timer},
};

use riscv_rt::InterruptNumber;

struct IObox {
    addr: u32,
    data: u32,
}

struct AddrMbx {
    stat: u32,
    ctrl: u32,
    ib: IObox,
    ob: IObox,
}

struct AddrSim {
    start: u32,
    stop: u32,
    prescaler: u32,
    loadfactor: u32,
    seed: u32,
}

struct AddrTask {
    period: u32,
    deadline: u32,
    ack: u32,
}

struct AddrTaskSet {
    mbx: AddrTask,
    update: [AddrTask; 4],
    control: [AddrTask; 4],
    report: [AddrTask; 4],
}

const SIM: AddrSim = AddrSim {
    start: 0x0100_0000,
    stop: 0x0100_0001,
    prescaler: 0x0100_0002,
    loadfactor: 0x0100_0003,
    seed: 0x0100_0004,
};

const MAILBOX: AddrMbx = AddrMbx {
    stat: 0x0003_0000,
    ctrl: 0x0003_0004,
    ib: IObox {
        addr: 0x0003_000C,
        data: 0x0003_0010,
    },
    ob: IObox {
        addr: 0x0003_0014,
        data: 0x0003_0018,
    },
};

const fn get_task_addr(idx: u32) -> AddrTask {
    let base = 0x0200_0000 + idx * 0x1_0000;
    let per_offs = 0x0;
    let dl_offs = 0x1;
    let ack_offs = 0x2;

    AddrTask {
        period: base + per_offs,
        deadline: base + dl_offs,
        ack: base + ack_offs,
    }
}

const TASKS: AddrTaskSet = AddrTaskSet {
    mbx: get_task_addr(0),
    update: [
        get_task_addr(1),
        get_task_addr(2),
        get_task_addr(3),
        get_task_addr(4),
    ],
    control: [
        get_task_addr(5),
        get_task_addr(6),
        get_task_addr(7),
        get_task_addr(8),
    ],
    report: [
        get_task_addr(9),
        get_task_addr(10),
        get_task_addr(11),
        get_task_addr(12),
    ],
};

const I2C_M0_ADDR: u8 = 0x10;
const I2C_M1_ADDR: u8 = 0x11;
const I2C_M2_ADDR: u8 = 0x12;
const I2C_M3_ADDR: u8 = 0x13;

const CTRL_TASK_PER_US: u32 = 2000;

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
const DL_CTRL: u32 = 1000;
const DL_UPD: u32 = 2000;
const DL_REP: u32 = 2000;

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

// Infer task period from DL and LF
const MBX_PER: u32 = 5 * DL_MBX + (4 * (100 - LF));

// NOTE: 1 x i2c 8-bit read-write (1 addr+1 read+1 addr+1 write)
// blocks for around 140 us (measured from wave)

const PRIO_CTRL: u8 = 4;
const PRIO_MAIL: u8 = 5;
const PRIO_UPD: u8 = 3;
const PRIO_REP: u8 = 1;

static mut MAIL: [u32; 4] = [0, 0, 0, 0];

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("\n\r### Starting rt_prof (bare-metal) benchmark ###\n\r");
    let _i2c = I2c::init(4);

    assert!(MBX_PER > DL_MBX, "Mailbox task period too short!\n\r");
    assert!(
        CTRL_TASK_PER_US > DL_CTRL,
        "Control task period too short!\n\r"
    );

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

    timers[0].set_period(CTRL_TASK_PER_US.micros());
    timers[1].set_period(CTRL_TASK_PER_US.micros());
    timers[2].set_period(CTRL_TASK_PER_US.micros());
    timers[3].set_period(CTRL_TASK_PER_US.micros());

    let mut mtimer = MTimer::instance().into_oneshot();

    sprintln!("Configuration and parameters:");
    sprintln!(" - Runtime (ms)          : {}", SIM_PARAMS.hyperperiod_ms);
    sprintln!(" - Sim env prescaler     : {}", PS);
    sprintln!(" - Randomization seed    : 0x{:X}", SEED);
    sprintln!(" - Load factor  [0-100]  : {}", LF);
    sprintln!(" - Update task DL   (us) : {}", DL_UPD);
    sprintln!(" - Report task DL   (us) : {}", DL_REP);
    sprintln!(" - Control task DL  (us) : {}", DL_CTRL);
    sprintln!(" - Control period   (us) : {}", CTRL_TASK_PER_US);
    sprintln!(" - Mailbox task DL  (us) : {}", DL_MBX);
    sprintln!(" - Mailbox period   (us) : {}", MBX_PER);
    sprintln!("");

    send_letter(TASKS.mbx.deadline, DL_MBX);
    send_letter(TASKS.update[0].deadline, DL_UPD);
    send_letter(TASKS.update[1].deadline, DL_UPD);
    send_letter(TASKS.update[2].deadline, DL_UPD);
    send_letter(TASKS.update[3].deadline, DL_UPD);
    wait_outbox_empty();
    send_letter(TASKS.control[0].deadline, DL_CTRL);
    send_letter(TASKS.control[1].deadline, DL_CTRL);
    send_letter(TASKS.control[2].deadline, DL_CTRL);
    send_letter(TASKS.control[3].deadline, DL_CTRL);
    wait_outbox_empty();
    send_letter(TASKS.report[0].deadline, DL_REP);
    send_letter(TASKS.report[1].deadline, DL_REP);
    send_letter(TASKS.report[2].deadline, DL_REP);
    send_letter(TASKS.report[3].deadline, DL_REP);
    wait_outbox_empty();

    send_letter(SIM.prescaler, PS);
    send_letter(SIM.loadfactor, LF);
    wait_outbox_empty();

    send_letter(SIM.start, 0x0);

    timers.iter_mut().for_each(Periodic::start);
    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    unsafe { riscv::interrupt::enable() };

    loop {
        wfi();
    }
}

#[inline]
fn wait_outbox_empty() {
    while (mmio::read_u32(MAILBOX.stat as usize) & (1 << 2)) == 0 {}
}

#[inline]
fn send_letter(addr: u32, data: u32) {
    mmio::write_u32(MAILBOX.ob.addr as usize, addr);
    mmio::write_u32(MAILBOX.ob.data as usize, data);
    // send letter
    mmio::write_u32(MAILBOX.ctrl as usize, 0x1);
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
#[allow(non_snake_case)]
fn MachineTimer() {
    riscv::interrupt::disable();

    // Terminate scoreboard
    send_letter(SIM.stop, 0x1);

    // Delay to let outbox clear
    for _ in 1..101 {
        nop();
    }

    let instret = riscv::register::minstret::read64();
    let active_time_cc = riscv::register::mcycle::read64();

    sprintln!("\n\rInstructions retired: {instret}, cycles: {active_time_cc}");

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
#[allow(non_snake_case)]
fn Timer0Cmp() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0, 0];

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.read(I2C_M0_ADDR, &mut rbuf); // Read motor status
    unsafe { riscv::interrupt::enable() };

    let _rdata = u16::from_le_bytes(rbuf);

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.write(I2C_M0_ADDR, &[0x10, 0x67]); // Write to motor
    unsafe { riscv::interrupt::enable() };

    // acknowledge this task
    ack_task(TASKS.control[0].ack);
    // Pend report task (HW)
    pend_irq(Interrupt::Ext0);
    // Pend report task (TB)
    pend_task(TASKS.report[0].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Cmp)]
#[allow(non_snake_case)]
fn Timer1Cmp() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.read(I2C_M1_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let _rdata = u8::from_le_bytes(rbuf);

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.write(I2C_M1_ADDR, &[0x11]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.control[1].ack);
    pend_irq(Interrupt::Ext1);
    pend_task(TASKS.report[1].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Cmp)]
#[allow(non_snake_case)]
fn Timer2Cmp() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];
    riscv::interrupt::disable();
    unsafe { I2c::instance() }.read(I2C_M2_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let _rdata = u8::from_le_bytes(rbuf);

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.write(I2C_M2_ADDR, &[0x12]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.control[2].ack);
    pend_irq(Interrupt::Ext2);
    pend_task(TASKS.report[2].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Cmp)]
#[allow(non_snake_case)]
fn Timer3Cmp() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_CTRL as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    let mut rbuf = [0];

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.read(I2C_M3_ADDR, &mut rbuf);
    unsafe { riscv::interrupt::enable() };

    let _rdata = u8::from_le_bytes(rbuf);

    riscv::interrupt::disable();
    unsafe { I2c::instance() }.write(I2C_M3_ADDR, &[0x13]);
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.control[3].ack);
    pend_irq(Interrupt::Ext3);
    pend_task(TASKS.report[3].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer0Ovf)]
#[allow(non_snake_case)]
fn Timer0Ovf() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.update[0].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer1Ovf)]
#[allow(non_snake_case)]
fn Timer1Ovf() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.update[1].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer2Ovf)]
#[allow(non_snake_case)]
fn Timer2Ovf() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.update[2].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Timer3Ovf)]
#[allow(non_snake_case)]
fn Timer3Ovf() {
    // raise mintthresh to task level
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_UPD as usize).into());
    // enable nesting manually
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.update[3].ack);

    // restore mintthresh
    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext0)]
#[allow(non_snake_case)]
fn Ext0() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.report[0].ack);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext1)]
#[allow(non_snake_case)]
fn Ext1() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.report[1].ack);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext2)]
#[allow(non_snake_case)]
fn Ext2() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.report[2].ack);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Ext3)]
#[allow(non_snake_case)]
fn Ext3() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_REP as usize).into());
    unsafe { riscv::interrupt::enable() };

    ack_task(TASKS.report[3].ack);

    bsp::register::mintthresh::write(last_mintthresh.into());
}

#[core_interrupt(bsp::interrupt::Interrupt::Mbx)]
#[allow(non_snake_case)]
fn Mbx() {
    let last_mintthresh = bsp::register::mintthresh::write((PRIO_MAIL as usize).into());
    unsafe { riscv::interrupt::enable() };

    for _ in 0..4 {
        // Read inbox
        let addr = mmio::read_u32(MAILBOX.ib.addr as usize);
        let data = mmio::read_u32(MAILBOX.ib.data as usize);

        unsafe {
            match addr {
                0x100 => MAIL[0] = data,
                0x101 => MAIL[1] = data,
                0x102 => MAIL[2] = data,
                0x103 => MAIL[3] = data,
                _ => sprintln!("Weird letter"),
            }
        }

        mmio::write_u32(MAILBOX.ctrl as usize, 0x0100_0000);
    }

    // IRQ clear
    mmio::write_u32(MAILBOX.ctrl as usize, 0x0002_0000);

    pend_task(TASKS.update[0].ack);
    pend_irq(Interrupt::Timer0Ovf);

    pend_task(TASKS.update[1].ack);
    pend_irq(Interrupt::Timer1Ovf);

    pend_task(TASKS.update[2].ack);
    pend_irq(Interrupt::Timer2Ovf);

    pend_task(TASKS.update[3].ack);
    pend_irq(Interrupt::Timer3Ovf);

    // Ack MBX task
    ack_task(TASKS.mbx.ack);

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
