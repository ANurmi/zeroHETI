#include <stdint.h>

void write_u8 (uint8_t addr, uint8_t wdata) {
  *(uint8_t*)(addr) = wdata;
}

void write_u16 (uint16_t addr, uint16_t wdata) {
  *(uint16_t*)(addr) = wdata;
}

void write_u32 (uint32_t addr, uint32_t wdata) {
  *(uint32_t*)(addr) = wdata;
}

uint8_t read_u8 (uint32_t addr) {
  return *(uint8_t*)(addr);
}

uint16_t read_u16 (uint32_t addr) {
  return *(uint16_t*)(addr);
}

uint32_t read_u32 (uint32_t addr) {
  return *(uint32_t*)(addr);
}
