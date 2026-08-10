//! Shared harness for the CLIC architecture tests.
#![no_std]
// Each test binary uses a different subset of the helpers, so unused-function
// warnings are expected noise here.
#![allow(dead_code)]
use bsp::{
    CPU_FREQ_HZ,
    apb_uart::ApbUart,
    clic::{Clic, Polarity, Trig, intattr::Mode},
    mtimer::MTimer,
    register::{mintstatus, mintthresh, mnxti},
    sprintln, tb,
};
use riscv_types::InterruptNumber;

/// Initialize the CLIC and the UART for a test: clear stale per-hart state,
/// enable full 8-bit interrupt levels, and configure the (real, `FULL_UART`)
/// APB UART so `sprintln!` does not hang polling for THRE.
pub fn init_arch_test() {
    // HACK: clear mintstatus, required for zeroHETI
    unsafe { mintstatus::write(0usize.into()) };
    // Set level bits to 8
    Clic::smclicconfig().set_mnlbits(8);
    mintthresh::write(0usize.into());
    // The verilated TB instantiates the real apb_uart (FULL_UART), which needs
    // a configured baud divisor before LSR.THRE ever re-asserts.
    let _serial = ApbUart::init(CPU_FREQ_HZ, 115_200);
}

/// Configure `irq` with a single level and otherwise the common defaults:
/// edge-triggered, positive polarity, selective hardware vectoring, M-mode,
/// enabled.
///
/// The default uses SHV so that `#[core_interrupt]` handlers are reached on
/// the standard vectored path (with `v-trap`, a non-SHV interrupt routes to
/// `DefaultHandler` instead).
pub fn setup_irq_lvl(irq: impl InterruptNumber, level: u8) {
    setup_irq_full(
        irq,
        level,
        Trig::Edge,
        Polarity::Pos,
        true,
        Mode::Machine,
        true,
    );
}

/// Configure `irq` fully.
pub fn setup_irq_full(
    irq: impl InterruptNumber,
    level: u8,
    trig: Trig,
    pol: Polarity,
    shv: bool,
    mode: Mode,
    ie: bool,
) {
    Clic::attr(irq).set_mode(mode);
    Clic::attr(irq).set_trig(trig);
    Clic::attr(irq).set_polarity(pol);
    Clic::attr(irq).set_shv(shv);
    Clic::ctl(irq).set_level(level);
    if ie {
        unsafe { Clic::ie(irq).enable() };
    } else {
        Clic::ie(irq).disable();
    }
    // Start from a clean slate for the line under test
    unsafe { Clic::ip(irq).unpend() };
}

/// Pend `irq` through the CLIC `clicintip` register (software-generated).
pub fn pend(irq: impl InterruptNumber) {
    unsafe { Clic::ip(irq).pend() };
}

/// Unpend `irq` through the CLIC `clicintip` register.
pub fn unpend(irq: impl InterruptNumber) {
    unsafe { Clic::ip(irq).unpend() };
}

/// Check whether `irq` is currently pending in the CLIC.
pub fn is_pending(irq: impl InterruptNumber) -> bool {
    unsafe { Clic::ip(irq).is_pending() }
}

/// Enable the per-interrupt enable bit (`clicintie`) for `irq`.
pub fn enable_ie(irq: impl InterruptNumber) {
    unsafe { Clic::ie(irq).enable() };
}

/// Disable the per-interrupt enable bit (`clicintie`) for `irq`.
pub fn disable_ie(irq: impl InterruptNumber) {
    Clic::ie(irq).disable();
}

/// Program the machine interrupt-level threshold.
pub fn set_thresh(v: u8) {
    mintthresh::write((v as usize).into());
}

/// Read the machine interrupt-level threshold.
pub fn get_thresh() -> u8 {
    mintthresh::read().bits() as u8
}

/// Enable global machine interrupts (`mstatus.mie = 1`).
pub fn enable_mie() {
    unsafe { bsp::riscv::interrupt::enable() };
}

/// Disable global machine interrupts (`mstatus.mie = 0`).
pub fn disable_mie() {
    bsp::riscv::interrupt::disable();
}

/// Read the raw value of the `mnxti` CSR.
pub fn read_mnxti() -> usize {
    mnxti::read().bits()
}

/// Read the machine interrupt level of the active handler
/// (`mintstatus.mil`).
pub fn read_mintstatus_mil() -> usize {
    mintstatus::read().mil()
}

/// A busy-polling deadline based on the `mtime` counter.
///
/// The `mtimecmp` is left at `u64::MAX` so no `MachineTimer` interrupt is ever
/// raised, keeping the watchdog side-effect free (it only counts).
pub struct Deadline {
    deadline: u64,
}

impl Deadline {
    /// Start a deadline `ms` milliseconds from now (polling `mtime`).
    pub fn start(ms: u64) -> Self {
        let mut t = MTimer::instance();
        // Counter starts at 0, counting enabled, mtimecmp = u64::MAX (no IRQ)
        t.reset();
        t.enable();
        let ticks = (ms as u64) * (CPU_FREQ_HZ as u64) / 1000;
        Self { deadline: ticks }
    }

    /// Returns `true` once the deadline has passed.
    pub fn expired(&self) -> bool {
        MTimer::instance().counter() >= self.deadline
    }

    /// Busy-wait while `cond()` holds, but abort (test failure) as soon as the
    /// deadline passes.
    #[cfg(feature = "rtl-tb")]
    pub fn spin_while(&self, cond: impl Fn() -> bool) {
        while cond() {
            if self.expired() {
                sprintln!("[FAIL] deadline expired");
                tb::signal_fail(None);
                loop {}
            }
        }
    }

    /// Busy-wait for the whole window. Used to give an expected interrupt a
    /// chance to fire (and to let a spuriously pending line be claimed).
    #[cfg(feature = "rtl-tb")]
    pub fn spin_full(&self) {
        while !self.expired() {
            core::hint::spin_loop();
        }
    }
}

/// Accumulate a pass/fail check result.
pub fn check(failures: &mut u32, label: &str, cond: bool) {
    if cond {
        sprintln!("  [OK]   {label}");
    } else {
        sprintln!("  [FAIL] {label}");
        *failures += 1;
    }
}

/// Finish the test, signaling pass/fail through the simulation backdoor.
#[cfg(feature = "rtl-tb")]
pub fn finish(failures: u32, label: &str) -> ! {
    if failures == 0 {
        sprintln!("[{label}] PASSED");
        tb::signal_pass(None);
    } else {
        sprintln!("[{label}] FAILED: {failures} check(s) failed");
        tb::signal_fail(None);
    }
    loop {}
}
