#![no_std]
#![no_main]

pub mod mailbox;

#[cfg(feature = "intc-clic")]
use bsp::clic::{Clic, Polarity, Trig};
use riscv_rt::InterruptNumber;

pub const UART_BAUD: u32 = if cfg!(feature = "rtl-tb") {
    1_500_000
} else {
    115_200
};

/// Setup `irq` for use with some basic defaults
///
/// Copy and customize this function if you need more involved configurations.
pub fn setup_irq(irq: impl InterruptNumber, level: u8) {
    log::debug!("Set up IRQ (id = {})", irq.number());
    #[cfg(feature = "intc-clic")]
    {
        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(level);
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-hetic")]
    {
        use bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level_prio(level);
        Hetic::line(irq.number()).enable();
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

/// Setup `irq` for use with some basic defaults
///
/// Copy and customize this function if you need more involved configurations.
#[cfg(feature = "intc-edfic")]
pub fn setup_irq_dl(irq: impl InterruptNumber, dl: u32) {
    log::debug!("Set up IRQ (id = {})", irq.number());
    use bsp::edfic::{Edfic, Pol, Trig};

    Edfic::line(irq.number()).set_pol(Pol::Pos);
    Edfic::line(irq.number()).set_trig(Trig::Edge);
    Edfic::line(irq.number()).enable();
    Edfic::line(irq.number()).set_dl(dl);
}

/// Tear down the IRQ configuration to avoid side-effects for further testing
///
/// Copy and customize this function if you need more involved configurations.
pub fn tear_irq(irq: impl InterruptNumber) {
    log::debug!("Tear down (id = {})", irq.number());
    #[cfg(feature = "intc-clic")]
    {
        Clic::ie(irq).disable();
        Clic::ctl(irq).set_level(0x0);
        Clic::attr(irq).set_shv(false);
        Clic::attr(irq).set_trig(Trig::Level);
        Clic::attr(irq).set_polarity(Polarity::Pos);
    }
    #[cfg(feature = "intc-hetic")]
    {
        use bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level_prio(0x0);
        Hetic::line(irq.number()).disable();
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::Edfic;

        Edfic::line(irq.number()).disable();
    }
}

pub fn pend_irq(irq: impl InterruptNumber) {
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::CLIC;
        unsafe { CLIC::ip(irq).pend() }
    }
    #[cfg(feature = "intc-hetic")]
    {
        use bsp::hetic::Hetic;
        Hetic::line(irq.number()).pend();
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::Edfic;
        Edfic::line(irq.number()).pend();
    }
}

#[macro_export]
macro_rules! print_reg_u32 {
    ($reg:expr) => {
        use bsp::read_u32;
        sprintln!("{:#x}: {} \"{}\"", $reg, read_u32($reg), stringify!($reg));
    };
}

/// Get the name of the current function
#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}
