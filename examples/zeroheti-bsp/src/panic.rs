/// Blinks the leds fast in a confused fashion (3 -> 1 -> 2 -> 0 -> 3)
#[panic_handler]
#[allow(unused_variables)]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // Initialize UART if not initialized
    if !unsafe { crate::apb_uart::UART_IS_INIT } {
        crate::apb_uart::ApbUart::init(crate::CPU_FREQ_HZ, crate::tb::DEFAULT_BAUD);
    }
    let mut uart = unsafe { crate::apb_uart::ApbUart::instance() };

    #[cfg(feature = "core-fmt")]
    {
        use embedded_io::Write;
        unsafe { write!(uart, "{}", info).unwrap_unchecked() };
    }
    #[cfg(feature = "ufmt")]
    write!(uart, "panic occurred");

    match () {
        #[cfg(feature = "rtl-tb")]
        () => {
            crate::tb::rtl_tb_signal_fail();
            loop {}
        }
        #[cfg(not(feature = "rtl-tb"))]
        () => crate::tb::blink_panic(),
    }
}
