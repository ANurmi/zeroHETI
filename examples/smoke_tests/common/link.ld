OUTPUT_ARCH(riscv)
SEARCH_DIR(.)
__DYNAMIC = 0;

MEMORY
{
  IMEM (rx ) : ORIGIN = 0x00001000, LENGTH = 0x4000
  DMEM (rwx) : ORIGIN = 0x00005000, LENGTH = 0x4000
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
  } > IMEM

  .data : ALIGN(4)
  {
    *(.data*);
  } > DMEM

  .sdata : ALIGN(4)
  {
    *(.sdata*);
  } > DMEM

  .bss (NOLOAD) :
  {
    *(.bss*)
    *(COMMON)
  } > DMEM
 
  .stack (NOLOAD) : 
  {
    . = ALIGN(4);
    stack = . + STACK_SIZE;
  } > DMEM

  _end = . ;
}
