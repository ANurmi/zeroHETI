//`define REG_TIMER                 2'b00
//`define REG_TIMER_CTRL            2'b01
//`define REG_CMP                   2'b10
//`define PRESCALER_STARTBIT        'd3
//`define PRESCALER_STOPBIT         'd5
//`define ENABLE_BIT                'd0

// TIMER_GROUP_BASE 0x2300
//
// TG0_TIMER 0x2300
// TG0_CTRL  0x2304
// TG0_CMP   0x2308
// TG1_TIMER 0x230C
// TG1_CTRL  0x2310
// TG1_CMP   0x2314

void timer_group_set_cmp(uint32_t idx, uint32_t cmp) {
  write_u32(TIMER_GROUP_BASE + (8 + (16*idx)), cmp);
}

void timer_group_start(uint32_t idx){
  set_bit_apb(TIMER_GROUP_BASE + (4 + (16*idx)), 0);
}

void timer_group_stop(uint32_t idx){
  clear_bit_apb(TIMER_GROUP_BASE + (4 + (16*idx)), 0);
}

// TODO: prescaler
