bits 64
global _start

section .rodata

  add#2: dq FUN#0
  main#3: dq FUN#2

section .text

_start:
  ; goto FUN#2
  jmp FUN#2


FUN#0:
  enter 0, 0

  ; register parameters as local variables
  push qword [rbp + 16]
  push qword [rbp + 24]

  ; t1 := x#0 + y#1
  mov rax, [rbp - 8]
  add rax, [rbp - 16]
  push qword rax

  ; ret t1
  mov rax, [rbp - 24]
  leave
  ret

FUN#2:
  enter 0, 0

  ; t3 := add#2(9, 2)
  push qword 2
  push qword 9
  call [add#2]
  push qword rax

  ; exit t3
  mov rax, 60
  mov rdi, [rbp - 8]
  syscall