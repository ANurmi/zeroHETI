.section .text

reset_handler:
  la x2, stack
	la t0, default_handler
	csrw mtvec, t0
  j main

  .section .vectors, "ax"
  .option norvc;
  .org 0x0
  jal x0, reset_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  .4byte isr_mtimer
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  .4byte trampoline_logging 
  jal x0, default_handler
  .4byte trampoline_control 
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler
  jal x0, default_handler

  //// reset vector
  .org 0x100
  jal x0, reset_handler

  .org 0x104

trampoline_logging:
	jal x1, isr_timer_logging
	mret

trampoline_control:
	#ifdef EABI
	addi sp, sp, -(4*9)
  sw ra, 0(sp)
  sw t0, 4(sp)
  sw a0, 8(sp)
  sw a1, 12(sp)
  sw a2, 16(sp)
  sw a3, 20(sp)
  sw t1, 24(sp)
  csrr t0, mepc
  csrr t1, mcause
  sw t0, 28(sp)
  sw t1, 32(sp)
	#else
	addi sp, sp, -(4 * 18)
	sw ra, 0(sp)
	sw t0, 4(sp)
	sw t1, 8(sp)
	sw t2, 12(sp)
	sw a0, 16(sp)
	sw a1, 20(sp)
	sw a2, 24(sp)
	sw a3, 28(sp)
	sw a4, 32(sp)
	sw a5, 36(sp)
	sw a6, 40(sp)
	sw a7, 44(sp)
	sw t3, 48(sp)
	sw t4, 52(sp)
	sw t5, 56(sp)
	sw t6, 60(sp)
  csrr t0, mepc
  csrr t1, mcause
  sw t0, 64(sp)
  sw t1, 68(sp)
	#endif
	jal x1, isr_timer_control
	#ifdef EABI
 	lw t0, 28(sp)
  lw t1, 32(sp)
  csrw mcause, t1
  csrw mepc, t0
  lw ra, 0(sp)
  lw t0, 4(sp)
  lw a0, 8(sp)
  lw a1, 12(sp)
  lw a2, 16(sp)
  lw a3, 20(sp)
  lw t1, 24(sp)
  addi sp, sp, (4 * 9)
	#else
 	lw t0, 64(sp)
  lw t1, 68(sp)
  csrw mcause, t1
  csrw mepc, t0
	lw ra, 0(sp)
	lw t0, 4(sp)
	lw t1, 8(sp)
	lw t2, 12(sp)
	lw a0, 16(sp)
	lw a1, 20(sp)
	lw a2, 24(sp)
	lw a3, 28(sp)
	lw a4, 32(sp)
	lw a5, 36(sp)
	lw a6, 40(sp)
	lw a7, 44(sp)
	lw t3, 48(sp)
	lw t4, 52(sp)
	lw t5, 56(sp)
	lw t6, 60(sp)
	addi sp, sp, (4 * 18)
	#endif
	mret

.balign 256
default_handler:
  li t0, 0x00000380
  li t1, 0x8BADC0DE
  sw t1, 0(t0)
  j default_handler
