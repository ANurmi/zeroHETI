//! Tests that all interrupts work as expected by raising them all, then
//! verifying this using the software dispatcher.
#![no_main]
#![no_std]
mod common;

use core::ptr::{self, addr_of, addr_of_mut};

use crate::common::{UART_BAUD, setup_irq};
use zeroheti_bsp::{
    CPU_FREQ_HZ,
    apb_uart::ApbUart,
    clic::{CLIC, Clic, InterruptNumber},
    interrupt::{CoreInterrupt, ExternalInterrupt},
    riscv,
    rt::entry,
    sprintln, tb,
};

/// Interrupts under testing
const CORE_IRQS: &[CoreInterrupt] = &[
    CoreInterrupt::MachineSoft,
    CoreInterrupt::MachineTimer,
    CoreInterrupt::MachineExternal,
];
const EXT_IRQS: &[ExternalInterrupt] = &[
    ExternalInterrupt::Timer0Ovf,
    ExternalInterrupt::Timer0Cmp,
    ExternalInterrupt::Timer1Ovf,
    ExternalInterrupt::Timer1Cmp,
    ExternalInterrupt::Timer2Ovf,
    ExternalInterrupt::Timer2Cmp,
    ExternalInterrupt::Timer3Ovf,
    ExternalInterrupt::Timer3Cmp,
    ExternalInterrupt::Uart,
    ExternalInterrupt::I2c,
    ExternalInterrupt::Mbx,
    ExternalInterrupt::Ext0,
    ExternalInterrupt::Ext1,
    ExternalInterrupt::Ext2,
    ExternalInterrupt::Ext3,
    //TODO: ???: enabling Nmi fails the test case
    //ExternalInterrupt::Nmi,
];

/// An array of 64 bits, one for each possible interrupt 0..64
static mut IRQ_RECVD: u64 = 0;

/// Example entry point
#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, UART_BAUD);
    //print_example_name!();

    // HACK: clear mintstatus, required for zeroHETI
    unsafe { zeroheti_bsp::register::mintstatus::write(0.into()) };

    // Set level bits to 8
    Clic::smclicconfig().set_mnlbits(8);

    for &irq in CORE_IRQS {
        sprintln!("Set up core {irq:?} (id = {})", irq.number());
        setup_irq(irq, 0x88);
    }
    for &irq in EXT_IRQS {
        sprintln!("Set up exti {irq:?} (id = {})", irq.number());
        setup_irq(irq, 0x88);
    }

    // Enable global interrupts
    sprintln!("mstatus.mie=1");
    unsafe { riscv::interrupt::enable() };

    // Raise each IRQ
    for &irq in CORE_IRQS.iter() {
        sprintln!("pend({})", irq.number());
        unsafe { CLIC::ip(irq).pend() };
    }
    for &irq in EXT_IRQS {
        sprintln!("pend({})", irq.number());
        unsafe { CLIC::ip(irq).pend() };
    }

    sprintln!("wait for acks...");
    // Busy wait for a while, or until all interrupts passed to make sure all interrupts have had time to be handled
    for _ in 0..2_000 {
        if CORE_IRQS.iter().all(|irq| {
            let bit: u64 = 0b1 << irq.number();
            (unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) & bit } == bit)
        }) && EXT_IRQS.iter().all(|irq: &ExternalInterrupt| {
            let bit: u64 = 0b1 << irq.number();
            (unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) & bit } == bit)
        }) {
            break;
        }
    }

    // Assert each interrupt was raised
    let mut failures = 0;
    for &irq in CORE_IRQS {
        let bit: u64 = 0b1 << irq.number();
        if unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) & bit } == bit {
            tb::signal_partial_ok!("{:?} = {}", irq, irq.number());
        } else {
            tb::signal_partial_fail!("{:?} = {}", irq, irq.number());
            failures += 1;
        }
    }
    for &irq in EXT_IRQS {
        let bit: u64 = 0b1 << irq.number();
        if unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) & bit } == bit {
            tb::signal_partial_ok!("{:?} = {}", irq, irq.number());
        } else {
            tb::signal_partial_fail!("{:?} = {}", irq, irq.number());
            failures += 1;
        }
    }

    // Save time, don't tear IRQs in RTL sim which gets exploded anyway
    #[cfg(not(feature = "rtl-tb"))]
    for &irq in CORE_IRQS {
        crate::common::tear_irq(irq);
    }
    #[cfg(not(feature = "rtl-tb"))]
    for &irq in EXT_IRQS {
        crate::common::tear_irq(irq);
    }

    if failures == 0 {
        tb::signal_pass(Some(&mut serial));
    } else {
        tb::signal_fail(Some(&mut serial));
    }
    loop {}
}

#[unsafe(export_name = "DefaultHandler")]
fn interrupt_handler() {
    // 8 LSBs of mcause must match interrupt id
    let irq_code = (riscv::register::mcause::read().bits() & 0xfff) as u16;

    // Record that the particular line was raised
    let mut val = unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) };
    val |= 0b1u64 << irq_code;
    unsafe { ptr::write_volatile(addr_of_mut!(IRQ_RECVD) as *mut _, val) };
}
