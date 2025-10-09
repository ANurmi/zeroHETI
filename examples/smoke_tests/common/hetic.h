// needs "addr_map.h", "mmio.h"

#define LINE_SIZE   0x2

#define IE_OFFSET   0x0
#define IP_OFFSET   0x1
#define TRIG_OFFSET 0x2
#define HETI_OFFSET 0x4
#define NEST_OFFSET 0x5
// bits [6:7] reserved
#define PRIO_OFFSET 0x8

void set_ip(uint8_t idx) {
  set_bit(HETIC_BASE + LINE_SIZE*idx, IP_OFFSET);
}
void clear_ip(uint8_t idx) {
  clear_bit(HETIC_BASE + LINE_SIZE*idx, IP_OFFSET);
}
void set_ie(uint8_t idx) {
  set_bit(HETIC_BASE + LINE_SIZE*idx, IE_OFFSET);
}
void clear_ie(uint8_t idx) {
  clear_bit(HETIC_BASE + LINE_SIZE*idx, IE_OFFSET);
}

// 0 -> EDGE, 1 -> LEVEL
void set_trig_type(uint8_t idx, uint8_t type) {
  if (type) {
    set_bit(HETIC_BASE + LINE_SIZE*idx, TRIG_OFFSET);
  } else {
    clear_bit(HETIC_BASE + LINE_SIZE*idx, TRIG_OFFSET);
  }
}

// 0 -> POS, 1 -> NEG
void set_trig_pol(uint8_t idx, uint8_t type) {
  if (type) {
    set_bit(HETIC_BASE + LINE_SIZE*idx, TRIG_OFFSET+1);
  } else {
    clear_bit(HETIC_BASE + LINE_SIZE*idx, TRIG_OFFSET+1);
  }
}

void set_prio (uint8_t idx, uint8_t prio) {
  write_u8(HETIC_BASE + LINE_SIZE*idx + 1, prio);
}
