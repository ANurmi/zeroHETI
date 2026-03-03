#!/bin/bash

OBJCOPY_32=riscv32-unknown-elf-objcopy
OBJCOPY_64=riscv64-unknown-elf-objcopy

OBJCOPY=$OBJCOPY_32

ELF=$1
OUTDIR=$2

IMEM_BYTES=$3
IMEM_BANKS=$4
IMEM_BASE=0x10000

DMEM_BYTES=$5
DMEM_BANKS=$6
DMEM_BASE=0x20000

echo "Formatting ELF $ELF for memories:"
echo "IMEM @$IMEM_BASE, $IMEM_BYTES bytes, $IMEM_BANKS banks"
echo "DMEM @$DMEM_BASE, $DMEM_BYTES bytes, $DMEM_BANKS banks"

if ! command -v $OBJCOPY_32 >/dev/null 2>&1
then
  if ! command -v $OBJCOPY_64 > /dev/null 2>&1
  then
    echo "No RISC-V objcopy on path, exiting."
    exit 1
  else
    OBJCOPY=OBJCOPY_64
  fi
fi

echo "Using objcopy: $OBJCOPY"
mkdir $OUTDIR/stims





