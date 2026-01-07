#include <stdint.h>

static inline void write_u8 (uint32_t addr, uint8_t wdata) {
  *(volatile uint8_t*)(addr) = wdata;
}

static inline void write_u16 (uint32_t addr, uint16_t wdata) {
  *(volatile uint16_t*)(addr) = wdata;
}

static inline void write_u32 (uint32_t addr, uint32_t wdata) {
  *(volatile uint32_t*)(addr) = wdata;
}

static inline uint8_t read_u8 (uint32_t addr) {
  return *(volatile uint8_t*)(addr);
}

static inline uint16_t read_u16 (uint32_t addr) {
  return *(volatile uint16_t*)(addr);
}

static inline uint32_t read_u32 (uint32_t addr) {
  return *(volatile uint32_t*)(addr);
}

static inline void set_bit(uint32_t addr, uint8_t pos){
  uint8_t write_val = read_u8(addr) | ((uint32_t)0x1 << pos);
  write_u8(addr, write_val);
}
static inline void clear_bit(uint32_t addr, uint8_t pos){
  uint8_t write_val = read_u8(addr) & ~((uint32_t)0x1 << pos);
  write_u8(addr, write_val);
}

// APB bus does not have byte enable, therefore we cannot
// use code that emits byte or halfword memory operations.
static inline void set_bit_apb(uint32_t addr, uint8_t pos){
  uint32_t write_val = read_u32(addr) | ((uint32_t)0x1 << pos);
  write_u32(addr, write_val);
}
static inline void clear_bit_apb(uint32_t addr, uint8_t pos){
  uint32_t write_val = read_u32(addr) & ~((uint32_t)0x1 << pos);
  write_u32(addr, write_val);
}
