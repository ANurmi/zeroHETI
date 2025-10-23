/* Author: Henri Lunnikivi <henri.lunnikivi@tuni.fi> */

ENTRY(_start)

MEMORY
{
  /* LENGTH = 0x5000 = 20K */
  IMEM (rx ) : ORIGIN = 0x5000, LENGTH = 0x8000
  /* LENGTH = 0x9000 = 36K */
  DMEM (rwx) : ORIGIN = 0xD000, LENGTH = 0x4000
}

/* Regions are setup like in link.ld for rt-ss written by Antti Nurmi. I didn't put any more thought into it. */

REGION_ALIAS("REGION_TEXT", IMEM);

REGION_ALIAS("REGION_RODATA", DMEM);
REGION_ALIAS("REGION_BSS", DMEM);
REGION_ALIAS("REGION_HEAP", DMEM);
REGION_ALIAS("REGION_STACK", DMEM);
REGION_ALIAS("REGION_DATA", DMEM);

/* Default interrupt trap entry point. When vectored trap mode is enabled,
   the riscv-rt crate provides an implementation of this function, which saves caller saved
   registers, calls the the DefaultHandler ISR, restores caller saved registers and returns.
*/
/*
PROVIDE(_start_DefaultHandler_trap = _start_trap);
*/

PROVIDE(_start_SupervisorSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorExternal_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineExternal_trap = _start_DefaultHandler_trap);
