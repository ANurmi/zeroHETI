proc print_sep {} {
  puts "----------------------------------------------------------------------"
}

proc print_file {} {
  print_sep
  puts "Running file in path:"
  puts [info script]
}


set FPGA_BOARD               $::env(FPGA_BOARD)
set FPGA_BOARD_CONFIG_SCRIPT $::env(FPGA_BOARD_CONFIG_SCRIPT)
