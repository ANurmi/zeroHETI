.section .text

reset_handler:
  la sp, stack
  la ra, postMain
  la t0, default_handler
  csrw mtvec, t0
  j main

.global postMain
postMain:
  li t0, 0x00000380
  li t1, 0x80000000
  or a0, a0, t1
  sw a0, 0(t0)

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

  //// reset vector
  .org 0x100
  jal x0, reset_handler

  .org 0x104

loop:
  j loop

.p2align 8
default_handler:
  li t0, 0x00000380
  li t1, 0x8BADC0DE
  sw t1, 0(t0)
  wfi
