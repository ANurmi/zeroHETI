//! Set 4 timers to trigger consecutively, each setting a flag. In main, check
//! for flags' presence until pass or failure on timeout.
#![no_main]
#![no_std]
mod common;

use core::{file, ptr};

use zeroheti_bsp::{
    CPU_FREQ_HZ, NOPS_PER_SEC,
    apb_uart::ApbUart,
    asm_delay,
    fugit::{ExtU32, ExtU64},
    interrupt::Interrupt,
    mmap::apb_timer::TIMER0_ADDR,
    mmio,
    mtimer::{MTimer, OneShot},
    rt::entry,
    sprintln,
    timer_group::Timer,
};

use crate::common::{init_intc, setup_irq, tear_irq};

#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", file!(), env!("RISCV_EXTS"));

    // TODO: include this check in BSP behind single function call
    const CFG_BASE_ADDR: usize = 0x0000_4000;
    let cfg = mmio::read_u32(CFG_BASE_ADDR + 4);
    let intc_edfic = (cfg & 0b1) == 0b1;
    if intc_edfic {
        #[cfg(feature = "intc-clic")]
        {
            panic!("Wrong interrupt controller, HW compiled to EDFIC");
        }
    } else {
        #[cfg(feature = "intc-edfic")]
        {
            panic!("Wrong interrupt controller, HW compiled to CLIC");
        }
    }

    init_intc();
    setup_irq(Interrupt::Timer0Cmp);
    setup_irq(Interrupt::Timer4Cmp);
    setup_irq(Interrupt::TqFull);
    setup_irq(Interrupt::TqNFull);
    setup_irq(Interrupt::MachineTimer);

    let mut mtimer = MTimer::instance();
    let mut timer = Timer::init::<TIMER0_ADDR>().into_periodic();

    timer.set_period(20u32.micros());
    timer.start();
    mtimer.start(100u64.micros());

    unsafe { riscv::interrupt::enable() };

    loop {
        asm_delay(NOPS_PER_SEC / 2);
        serial.write_str("[UART] tick\r\n");
    }
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer0Cmp)]
fn timer0() {
    // Stop the corresponding timer to avoid repeated timeouts
    unsafe { Timer::instance::<TIMER0_ADDR>() }.disable();
    tear_irq(Interrupt::Timer0Cmp);
}

#[zeroheti_bsp::core_interrupt(Interrupt::Timer4Cmp)]
fn timer4() {
    tear_irq(Interrupt::Timer4Cmp);
    zeroheti_bsp::tb::signal_pass(None);
}

#[zeroheti_bsp::core_interrupt(Interrupt::MachineTimer)]
fn timeout() {
    sprintln!("timeout");
    zeroheti_bsp::tb::signal_fail(None);
}
