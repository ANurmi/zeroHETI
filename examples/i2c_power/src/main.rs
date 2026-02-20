#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    hetic::Hetic,
    i2c::I2c,
    mmap::{apb_timer::*, i2c::*},
    mtimer::*,
    riscv::{self, asm::wfi},
    rt::entry,
    sprintln,
    tb::signal_pass,
    timer_group::*,
};
use core::arch::asm;
use fugit::{ExtU32, ExtU64};
use riscv::{InterruptNumber, interrupt::Interrupt};

const WRITE: u8 = 1;
const LAST: u8 = 1;
const NOT_LAST: u8 = 0;
const READ: u8 = 0;
const DT: f32 = 1.; // ms?
const KP: f32 = 0.066; // P factor
const KI: f32 = 0.1566; // I factor
const KD: f32 = 0.0668; // D factor
const R_NOMINAL: f32 = 16.2;

// Global variables
static mut I2C_READ_VAL: u8 = 0;
static mut I2C: Option<I2c<I2C_BASE>> = None;

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    let riscv_isa = core::env!("RISCV_ISA");

    sprintln!("[I2C Power control demo] ISA = {riscv_isa}");

    let mut i2c = I2c::<I2C_BASE>::init(4);

    let mut mtimer = MTimer::instance().into_oneshot();

    // MTIMER
    Hetic::line(7).set_level(0xff);
    Hetic::line(7).enable();

    // TG0CMP
    let mut tg0 = Timer::init::<TIMER0_ADDR>().into_periodic();
    tg0.set_period_offset(1000u32.micros(), 500u32.micros());
    Hetic::line(17).set_level(0x1);
    Hetic::line(17).enable();

    // TG1CMP
    let mut tg1 = Timer::init::<TIMER1_ADDR>().into_periodic();
    tg1.set_period(1000u32.micros());
    Hetic::line(19).set_level(0x7);
    Hetic::line(19).enable();

    // Start all timers
    mtimer.start(10u64.millis());
    //tg1.start();
    tg0.start();

    // Writing 1 to address 0 activates
    // power control simulation
    i2c.send_addr_frame(0, WRITE);
    i2c.send_data_frame(0x1, LAST);

    unsafe { I2C.replace(i2c) };

    unsafe {
        // clear instruction & cycle counters
        riscv::register::minstret::write(0);
        riscv::register::mcycle::write(0);
        riscv::register::minstreth::write(0);
        riscv::register::mcycleh::write(0);
        // Global enable
        riscv::interrupt::enable();
    }

    loop {
        wfi();
    }
}

#[bsp::core_interrupt(bsp::interrupt::CoreInterrupt::MachineTimer)]
unsafe fn MachineTimer() {
    let mut i2c = I2C.as_mut().unwrap();

    // Terminate power control simulation
    // by sending last I2C frame
    i2c.send_addr_frame(0, WRITE);
    i2c.send_data_frame(0x0, LAST);

    // capture minstret and mcycle (low) counters
    let minstret = riscv::register::minstret::read();
    let mcycle = riscv::register::mcycle::read();
    let minstreth = riscv::register::minstreth::read();
    let mcycleh = riscv::register::mcycleh::read();

    // Print out counters
    sprintln!(
        "[MTIME_ISR] mcycle {}",
        (((mcycleh as u64) << 32) | mcycle as u64)
    );
    sprintln!(
        "[MTIME_ISR] minstret {}",
        (((minstreth as u64) << 32) | minstret as u64)
    );

    // Write to polled debugger memory to end
    signal_pass(Some(&mut ApbUart::instance()));
    loop {}
}

#[bsp::nested_interrupt]
unsafe fn Timer0Cmp() {
    bsp::register::mintthresh::write(0x1.into());

    let i2c = I2C.as_mut().unwrap();
    i2c.send_addr_frame(0, READ);
    for _ in 0..10 {
        asm!("nop");
    }
    I2C_READ_VAL = i2c.recv_data_frame(LAST);
    riscv::interrupt::enable();
    let mcycle = riscv::register::mcycle::read();
    let mcycleh = riscv::register::mcycleh::read();

    let i2c_value = I2C_READ_VAL;
    sprintln!(
        "[LOGGER_ISR] I2C read: {i2c_value}, nominal: 50 mW, range: [45 mW - 55 mW], timestamp (mcycle): {}",
        (((mcycleh as u64) << 32) | mcycle as u64)
    );
    bsp::register::mintthresh::write(0x0.into());
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn custom_interrupt_handler() {
    sprintln!(
        "IRQ: {:#x?} = {:?}",
        riscv::register::mcause::read().code() & 0xfff,
        Interrupt::from_number(riscv::register::mcause::read().code() & 0xfff),
    );
}

static mut ERROR: f32 = 0.;
static mut ERROR_PREV: f32 = 0.;
static mut DERIVATIVE: f32 = 0.;
static mut INTEGRAL: f32 = 0.;

#[bsp::nested_interrupt]
unsafe fn Timer1Cmp() {
    let i2c = I2C.as_mut().unwrap();
    // capture minthresh
    let mintthresh_last = bsp::register::mintthresh::read();
    bsp::register::mintthresh::write(0xff.into());

    // read latest value
    i2c.send_addr_frame(0, READ);
    for _ in 0..10 {
        asm!("nop");
    }

    let i2c_read_val = i2c.recv_data_frame(LAST);
    I2C_READ_VAL = i2c_read_val;

    let p_measured = I2C_READ_VAL as f32;
    let p_target = 50f32;

    // Compute PID
    ERROR = p_target - p_measured;
    INTEGRAL += ERROR * DT;
    DERIVATIVE = (ERROR - ERROR_PREV) / DT;
    let p_ctrl = KP * ERROR + KI * INTEGRAL + KD * DERIVATIVE;
    ERROR_PREV = ERROR;

    let v_squared: f32 = p_ctrl * R_NOMINAL;
    let v_new = usqrt4(v_squared as u32) as u8;

    //sprintln!("V_new: {v_new}");

    // Write back computation result
    i2c.send_addr_frame(4, WRITE);
    i2c.send_data_frame(v_new, LAST);
    // capture mintthresh
    bsp::register::mintthresh::write(mintthresh_last.into());
}

fn usqrt4(val: u32) -> u32 {
    let mut a;
    let mut b;
    if val < 2 {
        return val;
    }
    a = 1255;

    b = val / a;
    a = (a + b) / 2;
    b = val / a;
    a = (a + b) / 2;
    b = val / a;
    a = (a + b) / 2;
    b = val / a;
    a = (a + b) / 2;

    a
}
