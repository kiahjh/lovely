bits 64
global _start

section .rodata

  get_three#0: dq FUN#0
  main#1: dq FUN#1

section .text

_start:
  ; goto FUN#1
  jmp FUN#1


FUN#0:
  enter 0, 0

  ; ret 3
  mov rax, 3
  leave
  ret

FUN#1:
  enter 0, 0

  ; t2 := get_three#0()
  call [get_three#0]
  push qword rax

  ; exit t2
  mov rax, 60
  mov rdi, [rbp - 8]
  syscall