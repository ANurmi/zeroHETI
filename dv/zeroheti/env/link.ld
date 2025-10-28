OUTPUT_ARCH( "riscv" )
ENTRY(rvtest_entry_point)

MEMORY
{
  IMEM (rx ) : ORIGIN = 0x5000, LENGTH = 0x8000
  DMEM (rwx) : ORIGIN = 0xD000, LENGTH = 0x4000
}

SECTIONS
{
  . = 0x00005000;
  .text.init : { *(.text.init) }
  . = ALIGN(0x1000);
  .tohost : { *(.tohost) }
  . = ALIGN(0x1000);
  .text : { *(.text) }
  . = 0x0000D000;
  .data : { *(.data) }
  .data.string : { *(.data.string)}
  .bss : { *(.bss) }
  _end = .;
}

