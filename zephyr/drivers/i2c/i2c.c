#include <stdint.h>
#include <stdlib.h>

#include <zephyr/kernel.h>
#include <zephyr/sys/sys_io.h>

#define I2C_BASE 0x00002200
#define I2C_CLK_PRESCALER I2C_BASE + 0
#define I2C_CTRL          I2C_BASE + 4
#define I2C_RX            I2C_BASE + 8
#define I2C_STATUS        I2C_BASE + 12
#define I2C_TX            I2C_BASE + 16
#define I2C_CMD           I2C_BASE + 20

static inline void i2c_set_prescaler(uint32_t ps){
  sys_write32(ps, I2C_CLK_PRESCALER);
}

uint32_t i2c_get_prescaler(void) {
  return sys_read32(I2C_CLK_PRESCALER);
}

static inline void i2c_core_enable(void){
  sys_set_bit(I2C_CTRL, 7u);
}

static inline void i2c_core_disable(void){
  sys_clear_bit(I2C_CTRL, 7u);
}

static inline uint8_t get_tip(void) {
  return sys_read32(I2C_STATUS) & (1u<<1);
}

static inline void i2c_set_cmd(uint8_t sta, uint8_t sto, uint8_t we, uint8_t ack, uint8_t ia){
  uint8_t r = (we == 0);
  uint8_t w = (we == 1);
  //                 STA    STO      RD      WR     ACK      IA
  uint32_t cmd = (sta<<7 | sto<<6 | r<<5 | w<<4 | ack<<3 | ia<<0 );
  sys_write32(cmd, I2C_CMD);
}

static inline void i2c_send_addr_frame(uint8_t addr, uint8_t we){
  uint8_t tx_addr = (addr << 1 | we);
  sys_write32(tx_addr, I2C_TX);
  i2c_set_cmd(1, 0, 1, 0, 0);
  while(get_tip());
}

static inline void i2c_send_data_frame(uint8_t data, uint8_t last){
  sys_write32(data, I2C_TX);
  i2c_set_cmd(0, last, 1, 0, 0);
  while(get_tip());
}

static inline uint8_t i2c_recv_data_frame(uint8_t last){
  i2c_set_cmd(0, last, 0, 0, 0);
  while(get_tip());
  return sys_read8(I2C_RX);
}

void i2c_init(uint32_t ps){
  i2c_set_prescaler(ps);
  i2c_core_enable();
}

void i2c_write_tx(uint8_t addr, uint8_t* tx_buf) {
  i2c_send_addr_frame(addr, 1);
  i2c_send_data_frame(0xAB, 1);
}

uint8_t* i2c_read_tx(uint8_t addr) {
  i2c_send_addr_frame(addr, 0);
  uint8_t* bytes = NULL;
  bytes = (uint8_t*)malloc(1 * sizeof(uint8_t));
  bytes[0] = i2c_recv_data_frame(1);
  return bytes;
}
