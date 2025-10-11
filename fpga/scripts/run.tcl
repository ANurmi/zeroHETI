# zeroHETI main FPGA implementation script
# Author(s): Antti Nurmi <antti.nurmi@tuni.fi>

set FPGA_TCL_DIR   $::env(FPGA_TCL_DIR)
set FPGA_BUILD_DIR $::env(FPGA_BUILD_DIR)
set PROJECT        $::env(PROJECT)

set FREQ_TARGET    $::env(FREQ_TARGET)

set DUT_TOP        zeroheti_top

source ${FPGA_TCL_DIR}/common.tcl
source ${FPGA_TCL_DIR}/pynq-z1.tcl

print_file

puts "\nStarting zeroHETI FPGA flow"
puts "Target Frequency: ${FREQ_TARGET} MHz\n"

create_project ${PROJECT} ${FPGA_BUILD_DIR} -force -part ${XLNX_PRT_ID}
set_property board_part ${XLNX_BRD_ID} [current_project]

auto_detect_xpm
source ${FPGA_TCL_DIR}/design_sources.tcl

set_property top ${DUT_TOP} [current_fileset]
