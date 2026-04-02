#include "verilated_fst_c.h"
#include "verilated.h"
#include "Vzeroheti_top_wrapper.h"
#include "Testbench.h"

#define xstr(s) str(s)
#define str(s) #s

// Add platform-specific overrides in this file
class TbZeroHeti: public Testbench<Vzeroheti_top_wrapper> {

    public:

        void print_logo(void) {
          printf("\n### zeroHETI on Verilator ###\n\n");
        }

        std::filesystem::path resolve_elf(std::string elf_name) {
          std::filesystem::path res = elf_name;
          if (elf_name.substr(elf_name.size() - 4) != ".elf") {
            res += ".elf";
          }

          if (!std::filesystem::exists(res)) {
            // naive path does not exist, look in build dir
            std::string repo_root = xstr(ZH_ROOT);
            std::filesystem::path bd = repo_root + "/build/sw/";
            std::filesystem::path full_path = bd.string() + res.string();
            if (std::filesystem::exists( full_path.string())) {
              res = full_path;
            } else {
              std::cerr << "ELF not found! Looked in:" << std::endl
                        << "1: " << res.string() << std::endl
                        << "2: " << bd.string() + res.string() << std::endl;
              std::exit(EXIT_FAILURE);
            }
          }
          return res;
        }

        void jtag_poll_addr(uint32_t addr) {
          uint32_t old = jtag_mm_read(addr);
          printf("[JTAG] Polling for activity on memory - address: 0x%08x, value 0x%08x\n", addr, old);
          uint32_t updated = 0;
          while (1){
            updated = jtag_mm_read(addr);
            if (updated != old) break;
          }
          printf("[JTAG] Detected update on memory      - address: 0x%08X, value 0x%08X\n", addr, updated);
        }
};
