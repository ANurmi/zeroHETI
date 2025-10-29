#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_top.h"
#include "Testbench.h"

#define STRING(s) #s

// Add platform-specific overrides in this file
class TbZeroHeti: public Testbench<Vzeroheti_top> {

    public:

        void print_logo(void) {
          printf("\n### zeroHETI on Verilator ###\n\n");
        }

        /*bool readback_test (uint32_t addr, uint32_t data){
            bool is_ok = true;
            uint32_t ref_data  = data;
            jtag_mm_write(addr, ref_data, 20, false);
            uint32_t read_data = jtag_mm_read(addr);
            if (ref_data != read_data){
                printf("Readback ERROR! Wrote: %x, read: %x \n", ref_data, read_data);
                is_ok = false;
            }
            return is_ok;
        }*/

        std::filesystem::path resolve_elf(std::string elf_name) {
          std::filesystem::path res = elf_name;
          if (elf_name.substr(elf_name.size() - 4) != ".elf") {
            res += ".elf";
          }

          if (!std::filesystem::exists(res)) {
            // naive path does not exist, look in build dir
            if (std::filesystem::exists( std::string(STRING(ZH_ROOT)) + "build/sw/" + res.string())) {
              res = "../build/sw/" + res.string();
            } else {
              std::cerr << "ELF not found" << std::endl;
              std::exit(EXIT_FAILURE);
            }
          }
          return res;
        }
/*
        void didactic_memtest () {

            printf("[TB] Running JTAG memory access test\n");
            printf("[JTAG] Test IMEM base address (0x0100_0000) access: ");
            if(readback_test(0x1000000, 0xDEADBEEF)){
                printf("OK\n");
            }

            printf("[JTAG] Test DMEM base address (0x0101_0000) access: ");
            if(readback_test(0x01010000, 0xBADC0FFE)){
                printf("OK\n");
            }
            
            printf("[JTAG] Test Debug base address (0x0102_0000) access: ");
            if(readback_test(0x01012000, 0xFEEDC0DE)){
                printf("OK\n");
            }
            printf("[JTAG] Test Staff Peripherals base address (0x0103_0000) access: ");
            if(readback_test(0x01013000, 0xBABEBABE)){
                printf("OK\n");
            }
            printf("[JTAG] Test Control Registers base address (0x0104_0000) access: ");
            if(readback_test(0x01013000, 0xFEEDBEEF)){
                printf("OK\n");
            }

            printf("[TB] JTAG test done\n");
        }

*/
};
