#ifndef I2C_H
#define I2C_H

uint32_t i2c_get_prescaler();

void i2c_init(uint32_t ps);

void i2c_write_tx(uint8_t addr, uint8_t* tx_buf);

uint8_t* i2c_read_tx(uint8_t addr);

#endif
