uint8_t get_tip(void) {
  return read_u32(I2C_STATUS) & (1u<<1);
}

void i2c_write_msg(uint8_t pld, uint8_t sta, uint8_t sto) {
  write_u32(I2C_TX, pld);
  //                 STA    STO      RD      WR     ACK      IA
  uint32_t cmd = (sta<<7 | sto<<6 | 0u<<5 | 1u<<4 | 1u<<3 | 0u<<0 );
  write_u32(I2C_CMD, cmd);
  while(get_tip());
}

void i2c_read_msg(uint8_t sta, uint8_t sto){
  //                  STA    STO      RD      WR     ACK      IA
  uint32_t cmd = ( sta<<7 | sto<<6 | 1u<<5 | 0u<<4 | 1u<<3 | 1u<<0 );
  write_u32(I2C_CMD, cmd);
  while(get_tip());
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
