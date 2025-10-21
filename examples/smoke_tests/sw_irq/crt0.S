.section .text

reset_handler:
  la x2, stack
  la ra, postMain 
  j main

.global postMain
postMain:
  li t0, 0x00000380
  li t1, 0x80000000
  or a0, a0, t1
  sw a0, 0(t0)
  wfi

.global changeStack
changeStack:
  mv sp, a2
  jr a1

.global machine_soft_trampoline
.func
machine_soft_trampoline:

  addi sp, sp, -(4*9)
  sw ra, (0*4)(sp)
  sw t0, (1*4)(sp)
  sw a0, (2*4)(sp)
  sw a1, (3*4)(sp)
  sw a4, (4*4)(sp)
  sw a5, (5*4)(sp)
  sw t1, (6*4)(sp)
  csrr t0, mepc
  csrr t1, mcause
  sw t0, (7*4)(sp)
  sw t1, (8*4)(sp)

  csrsi mstatus, 8 /* enable interrutps to allow nesting */
  call machine_soft_isr
  csrci mstatus, 8 /* disable interrutps */
  
  lw t0, (7*4)(sp)
  lw t1, (8*4)(sp)
  csrw mcause, t1
  csrw mepc, t0
  lw ra, (0*4)(sp)
  lw t0, (1*4)(sp)
  lw a0, (2*4)(sp)
  lw a1, (3*4)(sp)
  lw a4, (4*4)(sp)
  lw a5, (5*4)(sp)
  lw t1, (6*4)(sp)
  addi sp, sp, (4*9)
  mret
 
.endfunc

.section .vectors, "ax"
.option norvc;
.org 0x0
  j reset_handler
  .4byte default_handler
  .4byte default_handler
  .4byte machine_soft_trampoline
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler
  .4byte default_handler

  //// reset vector
  .org 0x100
  jal x0, reset_handler

  .org 0x104

loop:
  j loop

default_handler:
  mret
