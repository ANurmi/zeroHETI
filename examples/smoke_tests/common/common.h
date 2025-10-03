#include <stdint.h>

void write_u8 (uint32_t addr, uint8_t wdata) {
  *(uint8_t*)(addr) = wdata;
}

void write_u16 (uint32_t addr, uint16_t wdata) {
  *(uint16_t*)(addr) = wdata;
}

void write_u32 (uint32_t addr, uint32_t wdata) {
  *(uint32_t*)(addr) = wdata;
}

uint8_t read_u8 (uint32_t addr) {
  return *(volatile uint8_t*)(addr);
}

uint16_t read_u16 (uint32_t addr) {
  return *(uint16_t*)(addr);
}

uint32_t read_u32 (uint32_t addr) {
  return *(volatile uint32_t*)(addr);
}

void init_uart(uint32_t freq, uint32_t baud){

    uint32_t divisor = 0x700; // need to sort out compiler+div//freq / (baud << 4);

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


