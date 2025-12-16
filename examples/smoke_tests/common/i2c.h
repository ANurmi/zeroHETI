uint8_t get_tip(void) {
  return read_u32(I2C_STATUS) & (1u<<1);
}

void i2c_set_cmd(uint8_t sta, uint8_t sto, uint8_t we, uint8_t ack, uint8_t ia){
  uint8_t r = (we == 0);
  uint8_t w = (we == 1);
  //                 STA    STO      RD      WR     ACK      IA
  uint32_t cmd = (sta<<7 | sto<<6 | r<<5 | w<<4 | ack<<3 | ia<<0 );
  write_u32(I2C_CMD, cmd);
}

void i2c_send_addr_frame(uint8_t addr, uint8_t we){
  uint8_t tx_addr = (addr << 1 | we);
  write_u32(I2C_TX, tx_addr);
  i2c_set_cmd(1, 0, 1, 0, 0);
  while(get_tip());
}

void i2c_send_data_frame(uint8_t data, uint8_t last){
  write_u32(I2C_TX, data);
  i2c_set_cmd(0, last, 1, 0, 0);
  while(get_tip());
}

uint8_t i2c_recv_data_frame(uint8_t last){
  i2c_set_cmd(0, last, 0, 0, 0);
  while(get_tip());
  return read_u8(I2C_RX);
}

void i2c_set_prescaler(uint32_t val){
  write_u32(I2C_CLK_PRESCALER, val);
}

void i2c_core_enable(void){
  set_bit(I2C_CTRL, 7);
}

void i2c_core_disable(void){
  clear_bit(I2C_CTRL, 7);
}

void i2c_irq_enable(void){
  set_bit(I2C_CTRL, 6);
}

void i2c_irq_disable(void){
  clear_bit(I2C_CTRL, 6);
}
