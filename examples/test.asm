bits 64
global _start

section .rodata

  main#1: dq FUN#0

section .text

_start:
  ; goto FUN#0
  jmp FUN#0


FUN#0:
  enter 0, 0

  ; get_three#0 := FUN#1
  push qword FUN#1

  ; t2 := get_three#0()
  call [rbp - 8]
  push qword rax

  ; exit t2
  mov rax, 60
  mov rdi, [rbp - 16]
  syscall


FUN#1:
  enter 0, 0

  ; ret 3
  mov rax, 3
  leave
  ret