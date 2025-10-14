proc print_sep {} {
  puts "----------------------------------------------------------------------"
}

proc print_file {} {
  print_sep
  puts "Running file in path:"
  puts [info script]
}

proc set_checked {var} {
  if [info exists ::env(var)] {
    set var $::env(var);
  } else {
    puts "ERROR - Variable ${var} is not globally defined in Makefile!\n";
    return 1;
  }
}
