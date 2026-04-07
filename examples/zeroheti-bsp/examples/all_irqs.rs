//! Tests that all interrupts work as expected by raising them all, then
//! verifying this using the software dispatcher.
#![no_main]
#![no_std]
mod common;

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fintc-hetic, -Fintc-clic, -Fintc-edfic"
);

use core::ptr::{self, addr_of, addr_of_mut};
use riscv_pac::InterruptNumber;

use crate::common::{UART_BAUD, init_intc, pend_irq, setup_irq};
use zeroheti_bsp::{
    CPU_FREQ_HZ, apb_uart::ApbUart, interrupt::Interrupt, riscv, rt::entry, sprintln, tb,
};

/// Interrupts under testing
const IRQS: &[Interrupt] = &[
    Interrupt::MachineSoft,
    Interrupt::MachineTimer,
    // ???(BUG): MachineExternal broken on HETIC
    // Attempting to pend MachineExternal (line 11) on HETIC will freeze the
    // simulation or the application
    #[cfg(not(any(feature = "intc-hetic", feature = "intc-edfic")))]
    Interrupt::MachineExternal,
    Interrupt::Timer0Ovf,
    Interrupt::Timer0Cmp,
    Interrupt::Timer1Ovf,
    Interrupt::Timer1Cmp,
    Interrupt::Timer2Ovf,
    Interrupt::Timer2Cmp,
    Interrupt::Timer3Ovf,
    Interrupt::Timer3Cmp,
    Interrupt::Uart,
    Interrupt::I2c,
    Interrupt::Mbx,
    Interrupt::Ext0,
    Interrupt::Ext1,
    Interrupt::Ext2,
    Interrupt::Ext3,
    //TODO: ???: enabling Nmi fails the test case
    //Interrupt::Nmi,
];

/// An array of 64 bits, one for each possible interrupt 0..64
static mut IRQ_RECVD: u64 = 0;

/// Example entry point
#[entry]
fn main() -> ! {
    let mut serial = ApbUart::init(CPU_FREQ_HZ, UART_BAUD);
    //print_example_name!();

    init_intc();

    for &irq in IRQS {
        sprintln!("Set up IRQ {irq:?} (id = {})", irq.number());
        setup_irq(irq);
    }

    // Enable global interrupts
    sprintln!("mstatus.mie=1");
    unsafe { riscv::interrupt::enable() };

    // Raise each IRQ
    for &irq in IRQS {
        sprintln!("pend({})", irq.number());
        pend_irq(irq);
    }

    sprintln!("wait for acks...");
    // Busy wait for a while, or until all interrupts passed to make sure all interrupts have had time to be handled
    for _ in 0..2_000 {
        if IRQS.iter().all(|irq: &Interrupt| {
            let bit: u64 = 0b1 << irq.number();
            (unsafe { ptr::read_volatile(addr_of!(IRQ_RECVD) as *const _) & bit } == bit)
        }) {
            break;
        }
    }

    // Assert each interrupt was raised
    let mut failures = 0;
    for &irq in IRQS {
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
    for &irq in IRQS {
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
