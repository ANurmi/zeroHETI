use crate::apb_uart::ApbUartHal;

#[macro_export]
macro_rules! sprint {
    ($s:expr) => {{
        core::fmt::Write::write_fmt(&mut $crate::apb_uart::ApbUartHal::<{ $crate::mmap::uart::UART_BASE }>, format_args!($s)).unwrap()
    }};
    ($($tt:tt)*) => {{
        core::fmt::Write::write_fmt(&mut $crate::apb_uart::ApbUartHal::<{ $crate::mmap::uart::UART_BASE }>, format_args!($($tt)*)).unwrap()
    }};
}

#[macro_export]
macro_rules! sprintln {
    () => {{
        use $crate::sprint;
        sprint!("\r\n");
    }};
    // IMPORTANT use `tt` fragments instead of `expr` fragments (i.e. `$($exprs:expr),*`)
    ($($tt:tt)*) => {{
        use $crate::sprint;
        sprint!($($tt)*);
        sprint!("\r\n");
    }};
}

impl<const BASE_ADDR: usize> core::fmt::Write for ApbUartHal<BASE_ADDR> {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        ApbUartHal::write_str(self, s);
        Ok(())
    }
}
