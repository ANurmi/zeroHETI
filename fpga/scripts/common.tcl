proc print_sep {} {
  puts "----------------------------------------------------------------------"
}

proc print_file {} {
  print_sep
  puts "Running file in path:"
  puts [info script]
}

