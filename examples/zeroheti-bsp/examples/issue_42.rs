//! Demonstrate https://github.com/ANurmi/zeroHETI/issues/42
#![no_main]
#![no_std]
use riscv::{
    InterruptNumber,
    asm::{self, nop},
};
use riscv_rt::external_interrupt;
use zeroheti_bsp::{
    CPU_FREQ_HZ,
    apb_uart::ApbUart,
    asm_delay,
    clic::{Clic, Polarity, Trig},
    interrupt::{CoreInterrupt, ExternalInterrupt},
    rt::entry,
    sprintln, tb,
};

#[entry]
fn main() -> ! {
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);

    sprintln!("[{} ({})]", core::file!(), env!("RISCV_EXTS"));

    // HACK: clear mintstatus, required for zeroHETI
    unsafe { zeroheti_bsp::register::mintstatus::write(0.into()) };

    Clic::smclicconfig().set_mnlbits(8);

    let irq = ExternalInterrupt::Uart;

    // Setup IRQ with standard procedure
    Clic::attr(irq).set_trig(Trig::Edge);
    Clic::attr(irq).set_polarity(Polarity::Pos);
    Clic::attr(irq).set_shv(true);
    Clic::ctl(irq).set_level(0xff);

    // Enable global interrupts
    sprintln!("mstatus.mie=1");
    unsafe { riscv::interrupt::enable() };

    // STEP 1: pend the interrupt first
    unsafe { Clic::ip(irq).pend() }

    // STEP 2: enable the interrupt after pending
    unsafe { Clic::ie(irq).enable() };

    for _ in 0..1000 {
        asm::nop();
    }

    // Timeout
    tb::signal_fail(Some(&mut unsafe { ApbUart::instance() }));

    loop {
        asm_delay(1_000);
        sprintln!("[UART] Tick\r\n");
    }
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn custom_interrupt_handler() {
    let code = riscv::register::mcause::read().code() & 0xfff;
    if code <= CoreInterrupt::MAX_INTERRUPT_NUMBER {
        sprintln!(
            "Core IRQ: {:#x?} = {:?}",
            code,
            CoreInterrupt::from_number(riscv::register::mcause::read().code() & 0xfff)
        );
    } else {
        sprintln!(
            "Ext. IRQ: {:#x?} = {:?}",
            code,
            ExternalInterrupt::from_number(riscv::register::mcause::read().code() & 0xfff)
        )
    }

    if ExternalInterrupt::from_number(code).unwrap() == ExternalInterrupt::Uart {
        tb::signal_pass(Some(&mut unsafe { ApbUart::instance() }));
    } else {
        tb::signal_fail(Some(&mut unsafe { ApbUart::instance() }));
    }
}

/*
#[external_interrupt(ExternalInterrupt::Uart)]
fn handler() {
    tb::signal_pass(Some(&mut unsafe { ApbUart::instance() }));
}
*/
