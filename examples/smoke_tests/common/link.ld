OUTPUT_ARCH(riscv)
SEARCH_DIR(.)
__DYNAMIC = 0;

MEMORY
{
  IMEM (rx ) : ORIGIN = 0x10000, LENGTH = 0x4000
  DMEM (rwx) : ORIGIN = 0x20000, LENGTH = 0x2000
}

STACK_SIZE = 0x800;

REGION_ALIAS("REGION_TEXT", IMEM);
REGION_ALIAS("REGION_RODATA", DMEM);
REGION_ALIAS("REGION_DATA", DMEM);
REGION_ALIAS("REGION_BSS", DMEM);
REGION_ALIAS("REGION_STACK", DMEM);

SECTIONS
{
  .text : ALIGN(4)
  {
    KEEP(*(.vectors .vectors.*))
    _stext = .;
    *(.text*)
    _etext = .;
    . = ALIGN(4);
  } > REGION_TEXT

  .rodata : ALIGN(4)
  {
    *(.rodata)
    rodata_end = .;
    . = ALIGN(4);
  } > REGION_RODATA

  .data : ALIGN(4)
  {
    *(.data*);
    . = ALIGN(4);
  } > REGION_DATA

  .bss (NOLOAD) :
  {
    *(.bss*)
    *(COMMON)
    . = ALIGN(4);
  } > REGION_BSS

  .stack (NOLOAD) :
  {
    . = ALIGN(4);
    stack = . + STACK_SIZE;
  } > REGION_STACK
  . = ALIGN(4);

  _end = . ;
}
