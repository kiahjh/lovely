bits 64
global _start

section .rodata

  get_three#0: dq FUN#0
  main#3: dq FUN#1

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

  ; t2 := 4 + 5
  mov rax, 4
  add rax, 5
  push qword rax

  ; x#1 := t2
  push qword [rbp - 8]

  ; get_nine#2 := FUN#3
  push qword FUN#3

  ; t4 := get_three#0()
  call [get_three#0]
  push qword rax

  ; t5 := get_nine#2()
  call [rbp - 24]
  push qword rax

  ; t6 := t4 + t5
  mov rax, [rbp - 32]
  add rax, [rbp - 40]
  push qword rax

  ; exit t6
  mov rax, 60
  mov rdi, [rbp - 48]
  syscall


FUN#3:
  enter 0, 0

  ; ret x#1
  mov rax, [rbp - 16]
  leave
  ret