OUTPUT_ARCH( "riscv" )
ENTRY(rvtest_entry_point)

MEMORY
{
  IMEM (rx ) : ORIGIN = 0x10000, LENGTH = 0x8000
  DMEM (rwx) : ORIGIN = 0x20000, LENGTH = 0x4000
}

SECTIONS
{
  . = 0x00010000;
  .text.init : { *(.text.init) }
  .text : { *(.text) }
  . = 0x00020000;
  .tohost : { *(.tohost) }
  .data : { *(.data) }
  .data.string : { *(.data.string)}
  .bss : { *(.bss) }
  _end = .;
}

