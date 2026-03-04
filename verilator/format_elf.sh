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
# Same as above in dec +1
DMEM_START=16385

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
mkdir -p $OUTDIR/stims
STIMS=$OUTDIR/stims

$OBJCOPY $ELF -O binary $STIMS/tmp.bin
xxd -ps $STIMS/tmp.bin $STIMS/tmp.hex
tr -d '\n' < $STIMS/tmp.hex > $STIMS/tmp_nl.hex
sed -r 's/(.{8})/\1\n/g' < $STIMS/tmp_nl.hex > $STIMS/tmp_be.hex
sed ':a; /^.\{8\}$$/!s/$$/0/; ta' < $STIMS/tmp_be.hex > $STIMS/tmp_be_pad.hex
sed -E 's/(..)(..)(..)(..)/\4\3\2\1/' < $STIMS/tmp_be_pad.hex > $STIMS/tmp_le.hex
head -$(wc -w < $STIMS/tmp_le.hex) $STIMS/tmp_le.hex > $STIMS/elf.hex
rm $STIMS/tmp*

echo "Hex file produced in $STIMS/elf.hex"

IMEM_WORDS=$(($IMEM_BYTES/4))
DMEM_WORDS=$(($DMEM_BYTES/4))

IWORDS_BANK=$(($IMEM_WORDS/$IMEM_BANKS))
DWORDS_BANK=$(($DMEM_WORDS/$DMEM_BANKS))

head -$IMEM_WORDS    $STIMS/elf.hex > $STIMS/imem.hex
tail -n +$DMEM_START $STIMS/elf.hex > $STIMS/dmem.hex

for i in $(seq 0 $(($IMEM_BANKS-1)));
do
  BASE=$(($i * $IWORDS_BANK + 1))
  RANGE=$(($BASE + IWORDS_BANK))
  END=$(($RANGE + 1))
  sed -n "${BASE},${RANGE}p;${END}q" $STIMS/imem.hex > $STIMS/imem_$i.hex
done


for i in $(seq 0 $(($DMEM_BANKS-1)));
do
  BASE=$(($i * $DWORDS_BANK +1))
  RANGE=$(($BASE + DWORDS_BANK))
  END=$(($RANGE + 1))
  sed -n "${BASE},${RANGE}p;${END}q" $STIMS/dmem.hex > $STIMS/dmem_$i.hex
done

echo "Formatting done. Artifacts:"
echo "$STIMS/imem_{0..$(($IMEM_BANKS-1))}.hex"
echo "$STIMS/dmem_{0..$(($DMEM_BANKS-1))}.hex"
echo ""
