OUTPUT_ARCH(riscv)
SEARCH_DIR(.)
__DYNAMIC = 0;

MEMORY
{
  IMEM (rx ) : ORIGIN = 0x10000, LENGTH = 0x8000
  DMEM (rwx) : ORIGIN = 0x20000, LENGTH = 0x4000
}

STACK_SIZE = 0x1000;

SECTIONS
{
  .text : ALIGN(4)
  {
    KEEP(*(.vectors .vectors.*))
    _stext = .;
    *(.text*)
    _etext = .;
    *(.rodata*)
    . = ALIGN(4);
  } > IMEM

  .data : ALIGN(4)
  {
    *(.data*);
    . = ALIGN(4);
  } > DMEM

  .sdata : ALIGN(4)
  {
    *(.sdata*);
    . = ALIGN(4);
  } > DMEM

  .bss (NOLOAD) :
  {
    *(.bss*)
    *(COMMON)
    . = ALIGN(4);
  } > DMEM

  .stack (NOLOAD) :
  {
    . = ALIGN(4);
    stack = . + STACK_SIZE;
  } > DMEM

  _end = . ;
}
