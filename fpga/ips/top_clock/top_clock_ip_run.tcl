# ------------------------------------------------------------------------------
# top_clock_ip_run.tcl
#
# Author(s): Tom Szymkowiak <thomas.szymkowiak@tuni.fi>
# Date     : 04-dec-2023
#
# Description: TCL script to synthesise a clock wizard IP used for the top clock
# of the RT-SS subsystem
# ------------------------------------------------------------------------------

# Clear the console output
puts "\n---------------------------------------------------------"
puts "top_clock_ip_run.tcl - Starting..."
puts "---------------------------------------------------------\n"

source $::env(FPGA_TCL_DIR)/util.tcl
source $::env(FPGA_TCL_DIR)/env.tcl

set IP_PROJECT               $::env(IP_PROJECT)

# read in common and board specific variables
source ${FPGA_BOARD_CONFIG_SCRIPT}

# ------------------------------------------------------------------------------
# IP Configuration Variables
# ------------------------------------------------------------------------------

set INPUT_CLOCK_FREQ_MHZ    ${INPUT_OSC_FREQ_MHZ} ; # input to clocking wizard (read from scripts/<BOARD>.tcl)
set OUTPUT_CLOCK_1_FREQ_MHZ ${FREQ_TARGET}; # output from clocking wizard

# ------------------------------------------------------------------------------
# Synthesise IP
# ------------------------------------------------------------------------------

# IP for VCU118

create_project ${IP_PROJECT} . -part ${XLNX_PRT_ID}
set_property board_part ${XLNX_BRD_ID} [current_project]

create_ip -name clk_wiz -vendor xilinx.com -library ip -module_name ${IP_PROJECT}

set_property -dict [eval list CONFIG.PRIM_IN_FREQ "${INPUT_CLOCK_FREQ_MHZ}" \
                        CONFIG.NUM_OUT_CLKS {1} \
                        CONFIG.RESET_TYPE {ACTIVE_HIGH} \
                        CONFIG.RESET_PORT {reset} \
                        CONFIG.USE_LOCKED {true} \
                        CONFIG.CLKOUT1_REQUESTED_OUT_FREQ "${OUTPUT_CLOCK_1_FREQ_MHZ}" \
                       ] [get_ips ${IP_PROJECT}]

# Use differential clock input on VCU118
if {$FPGA_BOARD == "VCU118"} {
  set_property -dict [list CONFIG.PRIM_SOURCE {Differential_clock_capable_pin}] [get_ips ${IP_PROJECT}]
}

generate_target {all} \
    [get_files  ${IP_PROJECT}.srcs/sources_1/ip/${IP_PROJECT}/${IP_PROJECT}.xci]

create_ip_run \
    [get_files -of_objects [get_fileset sources_1] ${IP_PROJECT}.srcs/sources_1/ip/${IP_PROJECT}/${IP_PROJECT}.xci]

launch_run -jobs 8 ${IP_PROJECT}_synth_1
wait_on_run ${IP_PROJECT}_synth_1

puts "\n---------------------------------------------------------"
puts "top_clock_ip_run.tcl - Complete!"
puts "---------------------------------------------------------\n"

# ------------------------------------------------------------------------------
# End of Script
# ------------------------------------------------------------------------------
