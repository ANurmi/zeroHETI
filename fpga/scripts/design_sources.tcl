print_file

set RTIBEX_DIR       $::env(RTIBEX_DIR)
set AXI_DIR          $::env(AXI_DIR)
set APB_DIR          $::env(APB_DIR)
set OBI_DIR          $::env(OBI_DIR)
set REGIF_DIR        $::env(REGIF_DIR)
set COMMON_CELLS_DIR $::env(COMMON_CELLS_DIR)

set INCLUDE_PATHS " \
  ${RTIBEX_DIR}/vendor/lowrisc_ip/ip/prim/rtl \
  ${RTIBEX_DIR}/vendor/lowrisc_ip/dv/sv/dv_utils \
  ${REGIF_DIR}/include \
  ${OBI_DIR}/include \
  ${APB_DIR}/include \
  ${AXI_DIR}/include \
  ${COMMON_CELLS_DIR}/include \
";

set_property include_dirs ${INCLUDE_PATHS} [current_fileset]
set_property include_dirs ${INCLUDE_PATHS} [current_fileset -simset]

# get source files set by Bender
set SRC_FILES $::env(SRC_FILES)

add_files -norecurse -scan_for_includes ${SRC_FILES}

puts "Design sources read!"
print_sep
