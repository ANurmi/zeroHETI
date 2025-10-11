print_file

set BENDER_TARGETS "-t rtl -t fpga"
set BENDER_CMD     "exec bender script flist $BENDER_TARGETS"
set SRC_FILES [eval $BENDER_CMD]

add_files -norecurse -scan_for_includes ${SRC_FILES}

puts "Design sources read!"
print_sep
