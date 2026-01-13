#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

use bsp::embedded_io::Write;
use bsp::hetic::{Hetic, Pol, Trig};
use bsp::interrupt::CoreInterrupt;
use bsp::rt::{InterruptNumber, core_interrupt};
use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    interrupt::ExternalInterrupt,
    mmap::apb_timer::{TIMER0_ADDR, TIMER1_ADDR, TIMER2_ADDR, TIMER3_ADDR},
    mtimer::{self, MTimer},
    riscv::{self, asm::wfi},
    rt::entry,
    sprintln,
    tb::signal_pass,
    timer_group::{Periodic, Timer},
};
use core::arch::asm;
use fugit::ExtU32;
use more_asserts as ma;


#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    //sprintln!("[periodic_tasks ({})]", env!("RISCV_EXTS"));
    sprintln!("[periodic_tasks]");

    signal_pass(Some(&mut serial));
    loop {
        // Wait for interrupt
        wfi();
    }
}