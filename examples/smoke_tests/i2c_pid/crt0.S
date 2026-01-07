.section .text

reset_handler:
  la x2, stack
  j main

.global changeStack
changeStack:
  mv sp, a2
  jr a1

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

  //// reset vector
  .org 0x100
  jal x0, reset_handler

  .org 0x104

loop:
  j loop

default_handler:
  mret
