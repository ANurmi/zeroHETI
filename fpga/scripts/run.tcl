# zeroHETI main FPGA implementation script
# Author(s): Antti Nurmi <antti.nurmi@tuni.fi>

set FPGA_TCL_DIR   $::env(FPGA_TCL_DIR)
set FPGA_BUILD_DIR $::env(FPGA_BUILD_DIR)
set PROJECT        $::env(PROJECT)

source ${FPGA_TCL_DIR}/common.tcl

print_file

puts "\nStarting zeroHETI FPGA flow"
puts "Board: "
puts "Target Frequency: MHz\n"

create_project ${PROJECT} ${FPGA_BUILD_DIR} -force 
#-part ${PART_ID}
