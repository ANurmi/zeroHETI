#!/bin/bash

# Runs verilator using the ELF supplied as $1

# `-e` exit on non-zero exit status
# `-u` exit on variable expand failure
# `-x` echo commands
set -eux

PROJECT_ROOT=../../
BUILD_DIR=${PROJECT_ROOT}/build
SW_BUILD_DIR=${BUILD_DIR}/sw

# Ensure target dir exists
mkdir -p ${SW_BUILD_DIR}

# Copy binary ELF to where verilator can find it
cp $1 ${SW_BUILD_DIR}/$(basename $1).elf

XLEN=32

# Run verilator simulation with ELF
cd ${PROJECT_ROOT}
make simv TEST=$(basename $1) XLEN=${XLEN}
