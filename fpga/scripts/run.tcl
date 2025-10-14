# zeroHETI main FPGA implementation script
# Author(s): Antti Nurmi <antti.nurmi@tuni.fi>

set DUT_TOP           zeroheti_pynq

# Source common scripts
source $::env(FPGA_TCL_DIR)/util.tcl
source $::env(FPGA_TCL_DIR)/env.tcl
source $::env(FPGA_TCL_DIR)/pynq-z1.tcl

# Create project
print_file
puts "\nStarting zeroHETI FPGA flow"
puts "Target Frequency: ${FREQ_TARGET} MHz\n"

create_project ${PROJECT_NAME} ${FPGA_BUILD_DIR} -force -part ${XLNX_PRT_ID}
set_property board_part ${XLNX_BRD_ID} [current_project]

# auto-detect memory macros
auto_detect_xpm

# Set design sources
source ${FPGA_TCL_DIR}/design_sources.tcl
set_property verilog_define ${FPGA_SYN_DEFINES} [current_fileset];
set_property verilog_define ${FPGA_SIM_DEFINES} [current_fileset -simset];
add_files -fileset constrs_1 -norecurse ${FPGA_CONSTRAINT}
set_property top ${DUT_TOP} [current_fileset]

## Get IPs
# separate IPs into a list
if {[llength $FPGA_IP_LIST] != 0} {
  set FPGA_IP_LIST [split $FPGA_IP_LIST " "];
}
# add each synthesised IP to the project
foreach {IP} ${FPGA_IP_LIST} {
  puts "Adding ${IP} IP to project...";
  read_ip ${FPGA_IP_BUILD_DIR}/${IP}/${IP}.srcs/sources_1/ip/${IP}/${IP}.xci;
}

# Run synthesis

