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

  .section .vectors, "ax"
  .option norvc;
  .org 0x0
  j reset_handler
  .4byte default_handler
  .4byte default_handler
  .4byte machine_soft_isr
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
