// needs "mmio.h", "addr_map.h"

#define EN_OFFS 0
#define PS_OFFS 8

void start_mtimer() {
  set_bit_apb(MTIMER_CTRL, EN_OFFS);
}

void stop_mtimer() {
  clear_bit_apb(MTIMER_CTRL, EN_OFFS);
}

void set_mtimer_cmp(uint64_t cmp) {
  // Prevent unintented triggering
  write_u32(MTIMER_CMP_LO, 0xFFFFFFFF);
  write_u32(MTIMER_CMP_HI, ((uint32_t) (cmp >> 32)));
  write_u32(MTIMER_CMP_LO, ((uint32_t) cmp));
}

void set_mtimer_prescaler(uint8_t value) {
  // read original value, clear prescaler
  uint32_t rdata = read_u32(MTIMER_CTRL) & ((uint8_t)0 << PS_OFFS);
  // write back value with new prescaler set
  write_u32(MTIMER_CTRL, (rdata | (value << PS_OFFS)));
}
