bits 64
global _start

section .rodata

  add#2: dq FUN#0
  main#6: dq FUN#2

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

  ; subtract#5 := FUN#3
  push qword FUN#3

  ; t5 := subtract#5(3, 30)
  push qword 30
  push qword 3
  call [rbp - 8]
  add rsp, 16
  push qword rax

  ; t6 := add#2(20, t5)
  push qword [rbp - 16]
  push qword 20
  call [add#2]
  add rsp, 16
  push qword rax

  ; exit t6
  mov rax, 60
  mov rdi, [rbp - 24]
  syscall


FUN#3:
  enter 0, 0

  ; register parameters as local variables
  push qword [rbp + 16]
  push qword [rbp + 24]

  ; t4 := y#4 - x#3
  mov rax, [rbp - 16]
  sub rax, [rbp - 8]
  push qword rax

  ; ret t4
  mov rax, [rbp - 24]
  leave
  ret