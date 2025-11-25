OUTPUT_ARCH( "riscv" )
ENTRY(rvtest_entry_point)

SECTIONS
{
  . = 0x00000100;
  .text.init : { *(.text.init) }
  .text : { *(.text) }
  .tohost : { *(.tohost) }
  .data : { *(.data) }
  .data.string : { *(.data.string)}
  .bss : { *(.bss) }
  _end = .;
}

