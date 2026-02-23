use riscv_pac::InterruptNumber;
use zeroheti_bsp::sprintln;

pub const UART_BAUD: u32 = if cfg!(feature = "rtl-tb") {
    1_500_000
} else {
    115_200
};

pub fn init_intc() {
    #[cfg(feature = "intc-clic")]
    {
        use zeroheti_bsp::clic::Clic;
        // Set level bits to 8
        Clic::smclicconfig().set_mnlbits(8);
    }
    #[cfg(feature = "intc-hetic")]
    {
        // HETIC doesn't need global initialization
    }
    #[cfg(feature = "intc-edfic")]
    {
        // EDFIC doesn't need global initialization
    }
}

/// Setup `irq` for use with some basic defaults
///
/// Copy and customize this function if you need more involved configurations.
pub fn setup_irq(irq: impl InterruptNumber, level: u8) {
    #[cfg(feature = "intc-clic")]
    {
        use zeroheti_bsp::clic::{Clic, Polarity, Trig};

        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(level);
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-hetic")]
    {
        use zeroheti_bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level(level);
        Hetic::line(irq.number()).enable();
    }
    #[cfg(not(any(feature = "intc-clic", feature = "intc-hetic")))]
    sprintln!("Interrupt controller not set up. Please set intc-* feature");
}

/// Tear down the IRQ configuration to avoid side-effects for further testing
///
/// Copy and customize this function if you need more involved configurations.
#[allow(dead_code)]
pub fn tear_irq(irq: impl InterruptNumber) {
    sprintln!("Tear down (id = {})", irq.number());
    #[cfg(feature = "intc-clic")]
    {
        use zeroheti_bsp::clic::{Clic, Polarity, Trig};

        Clic::ie(irq).disable();
        Clic::ctl(irq).set_level(0x0);
        Clic::attr(irq).set_shv(false);
        Clic::attr(irq).set_trig(Trig::Level);
        Clic::attr(irq).set_polarity(Polarity::Pos);
    }
    #[cfg(feature = "intc-hetic")]
    {
        use zeroheti_bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level(0x0);
        Hetic::line(irq.number()).disable();
    }
    #[cfg(not(any(feature = "intc-clic", feature = "intc-hetic")))]
    sprintln!("Interrupt controller not set up. Please set intc-* feature");
}

pub fn pend_irq(irq: impl InterruptNumber) {
    #[cfg(feature = "intc-clic")]
    {
        use zeroheti_bsp::clic::CLIC;
        unsafe { CLIC::ip(irq).pend() }
    }
    #[cfg(feature = "intc-hetic")]
    {
        use zeroheti_bsp::hetic::Hetic;
        Hetic::line(irq.number()).pend();
    }
}
