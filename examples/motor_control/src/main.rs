#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

use bsp::{
    CPU_FREQ_HZ,
    apb_uart::*,
    i2c::I2c,
    interrupt::{CoreInterrupt, ExternalInterrupt},
    mmap::i2c::*,
    mmio::write_u32,
    mtimer::*,
    nested_interrupt,
    register::mintstatus::Mintstatus,
    riscv::{self},
    rt::entry,
    sprintln,
    tb::signal_pass,
};
use fugit::ExtU64;
use motor_control::*;

struct SimParams {
    hyperperiod_ms: u64,
}

const SIM_PARAMS: SimParams = SimParams { hyperperiod_ms: 1 };

// Global variables
static mut I2C: Option<I2c> = None;

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
    let riscv_isa = core::env!("RISCV_ISA");
    sprintln!("[Motor control demo] ISA = {riscv_isa}");

    let mut i2c = I2c::init(4);

    // HACK: clear mintstatus
    unsafe { bsp::register::mintstatus::write(0.into()) };

    // Set level bits to 8
    #[cfg(feature = "intc-clic")]
    bsp::clic::Clic::smclicconfig().set_mnlbits(8);
    setup_irq(CoreInterrupt::MachineTimer, 0x88);

    // Start the simulation
    let mut mtimer = MTimer::instance().into_oneshot();
    //mtimer.start(SIM_PARAMS.hyperperiod_ms.millis());
    mtimer.start(10.micros());
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
        //wfi();
    }
}

#[bsp::core_interrupt(CoreInterrupt::MachineTimer)]
unsafe fn MachineTimer() {
    signal_pass(None);
}
