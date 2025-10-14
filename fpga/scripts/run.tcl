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

# Elaboration 
synth_design -rtl -sfcu -name rtl_1

# Synthesis
set_msg_config -id {Timing 38-282} -new_severity {ERROR}

# Configure synthesis strategy
set_property STEPS.SYNTH_DESIGN.ARGS.FLATTEN_HIERARCHY none [get_runs synth_1];
# # Use single file compilation unit mode to prevent issues with import pkg::* statements in the codebase
set_property -name {STEPS.SYNTH_DESIGN.ARGS.MORE OPTIONS} -value -sfcu -objects [get_runs synth_1];
#
# # Launch synthesis
launch_runs synth_1;
wait_on_run synth_1;
open_run synth_1 -name netlist_1;
# # prevents need to run synth again
set_property needs_refresh false [get_runs synth_1];

# ------------------------------------------------------------------------------
# Run Place and Route (Implementation)
# ------------------------------------------------------------------------------

# Configure implementation strategy
set_property "steps.opt_design.args.directive" "Default" [get_runs impl_1]
set_property "steps.place_design.args.directive" "Default" [get_runs impl_1]
set_property "steps.route_design.args.directive" "Default" [get_runs impl_1]
set_property "steps.phys_opt_design.args.is_enabled" true [get_runs impl_1]
set_property "steps.phys_opt_design.args.directive" "Default" [get_runs impl_1]
set_property "steps.post_route_phys_opt_design.args.is_enabled" true [get_runs impl_1]
set_property "steps.post_route_phys_opt_design.args.directive" "Default" [get_runs impl_1]

# Launch implementation
launch_runs impl_1 -verbose;
wait_on_run impl_1

# ------------------------------------------------------------------------------
# Generate bitstream
# ------------------------------------------------------------------------------

launch_runs impl_1 -to_step write_bitstream
wait_on_run impl_1
#
open_run impl_1

