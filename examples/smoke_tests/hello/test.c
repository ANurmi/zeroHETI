#include "common.h"
#include "addr_map.h"

int main() {

  //write_u32(0xA000, 0xDEADBEEF);
  uint32_t test_0 = read_u32(0x5000);
  uint16_t test_1 = read_u16(0x5000);
  uint8_t  test_2 = read_u8(0x5000);

  return 0;
}
