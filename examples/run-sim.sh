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

xxd -ps ${SW_BUILD_DIR}/tmp.bin ${SW_BUILD_DIR}/tmp.hex
tr -d '\n' < ${SW_BUILD_DIR}/tmp.hex > ${SW_BUILD_DIR}/tmp_nl.hex
sed -r 's/(.{8})/\1\n/g' < ${SW_BUILD_DIR}/tmp_nl.hex > ${SW_BUILD_DIR}/tmp_be.hex
sed ':a; /^.\{8\}$$/!s/$$/0/; ta' < ${SW_BUILD_DIR}/tmp_be.hex > ${SW_BUILD_DIR}/tmp_be_pad.hex
sed -E 's/(..)(..)(..)(..)/\4\3\2\1/' < ${SW_BUILD_DIR}/tmp_be_pad.hex > ${SW_BUILD_DIR}/tmp_le.hex

# "trim" hex
head -$(wc -w < ${SW_BUILD_DIR}/tmp_le.hex) ${SW_BUILD_DIR}/tmp_le.hex > ${SW_BUILD_DIR}/$(basename $1).hex
rm ${SW_BUILD_DIR}/tmp*

# Run verilator simulation with ELF
cd ${PROJECT_ROOT}
make simv TEST=$(basename $1)
