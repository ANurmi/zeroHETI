#include <stdint.h>

void debug_signal_pass(void){
  *(uint32_t*)(0x0380) = 0x80000000;
}
