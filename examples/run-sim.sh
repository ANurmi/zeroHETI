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

# Make hex
TOOLCHAIN=riscv32-unknown-elf-
${TOOLCHAIN}objcopy ${SW_BUILD_DIR}/$(basename $1).elf -O binary ${SW_BUILD_DIR}/tmp.bin
xxd -R never -e ${SW_BUILD_DIR}/tmp.bin ${SW_BUILD_DIR}/tmp.hex
cut -c 11-45 ${SW_BUILD_DIR}/tmp.hex > ${SW_BUILD_DIR}/tmp_pruned.hex
sed -E 's/ /\n/g' ${SW_BUILD_DIR}/tmp_pruned.hex > ${SW_BUILD_DIR}/tmp_align.hex

# "trim" hex
head -$(wc -w < ${SW_BUILD_DIR}/tmp_align.hex) ${SW_BUILD_DIR}/tmp_align.hex > ${SW_BUILD_DIR}/$(basename $1).hex
rm ${SW_BUILD_DIR}/tmp*

# Run verilator simulation with ELF
cd ${PROJECT_ROOT}
make verilate simv TEST=$(basename $1)
