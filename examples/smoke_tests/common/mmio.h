#include <stdint.h>

void write_u8 (uint32_t addr, uint8_t wdata) {
  *(volatile uint8_t*)(addr) = wdata;
}

void write_u16 (uint32_t addr, uint16_t wdata) {
  *(volatile uint16_t*)(addr) = wdata;
}

void write_u32 (uint32_t addr, uint32_t wdata) {
  *(volatile uint32_t*)(addr) = wdata;
}

uint8_t read_u8 (uint32_t addr) {
  return *(volatile uint8_t*)(addr);
}

uint16_t read_u16 (uint32_t addr) {
  return *(volatile uint16_t*)(addr);
}

uint32_t read_u32 (uint32_t addr) {
  return *(volatile uint32_t*)(addr);
}

void set_bit(uint32_t addr, uint8_t pos){
  write_u8(addr, 0x1);
}
void clear_bit(uint32_t addr, uint8_t pos){

}

