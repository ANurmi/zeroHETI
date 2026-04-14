//! Power profiling test for zeroHETI

// run with
//```
// RUNTIME_MS=XXX TASK_PER_US=YYY cargo run --release -Frtl-tb -Fintc-clic --example power_prof
//```

#![no_main]
#![no_std]
mod common;

use crate::common::{UART_BAUD, init_intc, pend_irq, setup_irq};
use fugit::ExtU32;
use fugit::ExtU64;
use riscv::asm::wfi;
use zeroheti_bsp::{
    CPU_FREQ_HZ, apb_uart::ApbUart, interrupt::Interrupt, mmap::apb_timer::TIMER0_ADDR,
    mtimer::MTimer, nested_interrupt, rt::entry, sprintln, timer_group::Timer,
};

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

const TASK_PER_US: u32 = parse_u32(env!("TASK_PER_US"));
const RUNTIME_MS: u64 = parse_u32(env!("RUNTIME_MS")) as u64;

const PI: f32 = 3.14159;

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams {
    hyperperiod_ms: RUNTIME_MS,
};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    sprintln!("[UART] zeroHETI power profiling test");

    init_intc();
    setup_irq(Interrupt::MachineTimer);
    setup_irq(Interrupt::Timer0Cmp);

    // Print test params
    sprintln!("Simulation parameters:");
    sprintln!(" - Runtime     (ms): {}", RUNTIME_MS);
    sprintln!(" - Task period (us): {}", TASK_PER_US);
    sprintln!("### SIM START ###");

    let mut mtimer = MTimer::instance().into_oneshot();
    let timer = &mut [Timer::init::<TIMER0_ADDR>().into_periodic()];
    timer[0].set_period(TASK_PER_US.micros());
    timer[0].start();

    unsafe {
        // Clear instruction & cycle counters
        riscv::register::minstret::write(0);
        riscv::register::mcycle::write(0);
        riscv::register::minstreth::write(0);
        riscv::register::mcycleh::write(0);
    }

    mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());

    unsafe { riscv::interrupt::enable() };

    loop {
        wfi();
    }
}

#[nested_interrupt]
fn Timer0Cmp() {
    let time_f = riscv::register::mcycle::read64() as f32;
    sprintln!("mcycle × pi = {}", (time_f * PI) as u32);
}

#[nested_interrupt]
fn MachineTimer() {
    unsafe { riscv::interrupt::disable() };
    let active_time_cc = riscv::register::mcycle::read64();
    let total_time_cc: u64 = MTimer::instance().counter().into();

    sprintln!("### SIM END ###");

    sprintln!(
        " - Total time (cc): {}, active time (cc): {},",
        total_time_cc,
        active_time_cc
    );
    sprintln!(
        " - CPU utilization: {}%",
        (active_time_cc * 100) / total_time_cc
    );
    zeroheti_bsp::tb::rtl_tb_signal_ok();
}
