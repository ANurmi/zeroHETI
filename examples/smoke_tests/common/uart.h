
void init_uart(uint32_t freq, uint32_t baud){

    // TODO: fix compiler ABI
    uint32_t divisor = 0x4; //freq / (baud << 4);

    write_u8(UART_INTERRUPT_ENABLE, 0x00); // disable uart interrupt
    write_u8(UART_LINE_CONTROL, 0x80);     // Enable DLAB (set baud rate divisor)
    write_u8(UART_DLAB_LSB, divisor);         // divisor (lo byte)
    write_u8(UART_DLAB_MSB, (divisor >> 8) & 0xFF);  // divisor (hi byte)
    write_u8(UART_LINE_CONTROL, 0x03);     // 8 bits, no parity, one stop bit
    write_u8(UART_FIFO_CONTROL, 0xC7);     // Enable FIFO, clear them, with 14-byte threshold
    write_u8(UART_MODEM_CONTROL, 0x20);    // Autoflow mode
}

uint8_t is_transmit_empty(void) {
  return read_u8(UART_LINE_STATUS) & 0x20;
}

void write_serial (uint8_t a) {
  while (is_transmit_empty() == 0) ;
  write_u8(UART_THR, a);
}

void print_uart(const uint8_t *str){
    const uint8_t *cur = &str[0];
    while (*cur != '\0')
    {
        write_serial(*cur);
        ++cur;
    }
}

void bin_to_hex(uint8_t inp, uint8_t res[2]){   
    uint8_t inp_low = (inp & 0xf);
    uint8_t inp_high = ((inp >> 4) & 0xf);

    res[1] = inp_low < 10 ? inp_low + 48 : inp_low + 55;
    res[0] = inp_high < 10 ? inp_high + 48 : inp_high + 55;
}

void print_uart_u32(uint32_t val){
	for (int i = 3; i > -1; i--){
		uint8_t cur = (val >> (i*8)) & 0xff;
		uint8_t hex[2];
		bin_to_hex(cur, hex);
		write_serial(hex[0]);
		write_serial(hex[1]);
	}
}
