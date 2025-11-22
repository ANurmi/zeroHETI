
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
