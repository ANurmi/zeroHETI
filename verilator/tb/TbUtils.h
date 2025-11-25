
typedef struct {
    // assume 32-bit addressing for now
    uint32_t entry;
    uint32_t phoff;
    uint32_t shoff;
    uint16_t ehsize;
    uint16_t phnum;
    uint16_t phentsize;
    uint16_t shnum;
    uint16_t shentsize;
    uint16_t shstrndx;
} elf_hdr_t;

typedef struct {
    // assume 32-bit addressing for now
    uint32_t type;
    uint32_t offset;
    uint32_t vaddr;
    uint32_t paddr;
    uint32_t filesz;
    uint32_t memsz;
    uint32_t flags;
    uint32_t align;
} prog_hdr_t;

template<typename T>
uint32_t get_from_offset(std::string input_string, const uint32_t offs ) {
    uint32_t size   = sizeof(T);
    uint32_t result = 0;
    for (int i=size-1; i>=0; i--){
        printf("byte: %x, addr %x\n", input_string[offs+i], offs+i);
        result |= input_string[offs+i] << 8*i;
    }
    return result;
}

const std::string get_elf(const std::string path){
      std::fstream fs;
      std::string line;
      std::string concat = "";
        
      fs.open(path, std::ios::in);

      // Concatenate ELF contents to single string
      while (getline (fs, line)) {
          concat = concat + line;
      }
      fs.close();
      return concat;
}

elf_hdr_t parse_elf_hdr(std::string input_string) {
    //uint32_t test = get_from_offset<uint32_t>(input_string, 0x1C);
    //printf("%x\n", test);
    //std::exit(EXIT_SUCCESS);

    elf_hdr_t ehdr;

    // Check magic value, implicit ok branch
    if(!(input_string[0] == 0x7F && (input_string.substr(1,3) == "ELF"))) {
        std::cout << "[ELFLOAD] ERROR: ELF Format not OK" << std::endl;
    }

    //std::string bitwidth  = (input_string[4] == 0x1) ? "32" : "64";
    //std::string endianess = (input_string[5] == 0x1) ? "little" : "big";

    // TODO: all offsets statically based on 32-bit addresses
    ehdr.entry = get_from_offset<uint32_t>(input_string, 0x18);
    ehdr.phoff = get_from_offset<uint32_t>(input_string, 0x1C);
    ehdr.shoff = get_from_offset<uint32_t>(input_string, 0x20); 
    ehdr.phnum = get_from_offset<uint16_t>(input_string, 0x2C);
    ehdr.phentsize = get_from_offset<uint16_t>(input_string, 0x2A);
    ehdr.shnum = get_from_offset<uint16_t>(input_string, 0x30);
    ehdr.shentsize = get_from_offset<uint16_t>(input_string, 0x2E);
    ehdr.shstrndx = get_from_offset<uint16_t>(input_string, 0x32);
    return ehdr;
}

prog_hdr_t parse_pgr_hdr(std::string input_string, uint32_t offs) {

    prog_hdr_t phdr;

    phdr.type = get_from_offset<uint32_t>(input_string, offs);
    phdr.offset = get_from_offset<uint32_t>(input_string, offs + 0x4);
    phdr.vaddr = get_from_offset<uint32_t>(input_string, offs + 0x8);
    phdr.paddr = get_from_offset<uint32_t>(input_string, offs + 0xC);
    phdr.filesz = get_from_offset<uint32_t>(input_string, offs + 0x10);
    phdr.memsz = get_from_offset<uint32_t>(input_string, offs + 0x14);
    phdr.flags = get_from_offset<uint32_t>(input_string, offs + 0x18);
    phdr.align = get_from_offset<uint32_t>(input_string, offs + 0x1C);

    return phdr;

}

void check_path(std::string ElfPath) {
  std::cout << "[TB:init] ELF path: " << ElfPath << std::endl;
  if(!std::filesystem::exists(ElfPath)) {
    std::cout << "ELF not found!" << std::endl;
    std::exit(EXIT_FAILURE);
  }
}

void load_memory(const std::string ElfPath, std::unordered_map<uint32_t, uint32_t>& mem){
  check_path(ElfPath);
  const std::string elf_string = get_elf(ElfPath);
  elf_hdr_t e = parse_elf_hdr(elf_string);

  for (uint32_t i=0; i<e.phnum; i++) {
    const uint32_t prog_hdr_offs = e.phoff + e.phentsize*i;
    prog_hdr_t p = parse_pgr_hdr(elf_string, prog_hdr_offs);
    bool type_load = (p.type == 1);

    if (p.memsz != 0 & type_load) {
      printf("[TB:init] Writing LOAD section to 0x%08x\n", p.paddr);
      for (int j=0; j<p.filesz; j = j+4) {
        printf("%x\n", p.offset);
        const uint32_t addr = p.paddr + j;
        const uint32_t data = get_from_offset<uint32_t>(elf_string, p.offset+j);
        mem[addr] = data;
        printf("Address %08X Data %08X, offs %x \n", addr, data, p.offset+j);
      }
    }
  }

  std::cout << "TODO: load memory contents from elf" << std::endl;
}

void parse_signature(const std::string sig_path) {

  std::ifstream iFile(ZhRoot+"/build/verilator_build/memdump_tmp.hex");
  std::ofstream oFile(sig_path);

  std::string line;
  bool start = false;
  bool end   = false;

  if (iFile.is_open()){
    
    while (getline(iFile,line)){
      if (line == Canary) {
        if (!start) start = true;
        else end = true;
      }
      else if (start & !end) {
        oFile << line + "\n";
      }
    }
    iFile.close();
    oFile.close();
  }
}
