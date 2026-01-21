#include <stdint.h>

static void debug_signal_pass(void){
  *(uint32_t*)(0x0380) = 0x80000000;
}
