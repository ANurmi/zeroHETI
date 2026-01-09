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
  .4byte isr_timer_control
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
	call isr_timer_logging
	mret

.balign 256
default_handler:
  li t0, 0x00000380
  li t1, 0x8BADC0DE
  sw t1, 0(t0)
  j default_handler
