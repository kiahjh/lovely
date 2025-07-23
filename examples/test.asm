bits 64
global _start

section .rodata

  get_three#0: dq FUN#0
  main#2: dq FUN#1

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

  ; get_four#1 := FUN#2
  push qword FUN#2

  ; t3 := get_three#0()
  call [get_three#0]
  push qword rax

  ; t4 := get_four#1()
  call [rbp - 8]
  push qword rax

  ; t5 := t3 + t4
  mov rax, [rbp - 16]
  add rax, [rbp - 24]
  push qword rax

  ; exit t5
  mov rax, 60
  mov rdi, [rbp - 32]
  syscall


FUN#2:
  enter 0, 0

  ; ret 4
  mov rax, 4
  leave
  ret