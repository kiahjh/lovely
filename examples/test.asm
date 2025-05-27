global _start

section .text

_start:

  ; get_four#0 := FUN#0
  lea rbx, [FUN#0]

  ; main#2 := FUN#2
  lea rcx, [FUN#2]

  ; goto FUN#2
  jmp FUN#2

FUN#0:

  ; t1 := 2 + 2
  mov rdx, 2
  add rdx, 2

  ; ret t1
  mov rax, rdx
  ret

FUN#2:

  ; t3 := get_four#0()
  ; TODO: function arguments
  call rbx
  mov rsi, rax

  ; x#1 := t3
  mov rdi, rsi

  ; t4 := x#1 + 3
  mov r8, rdi
  add r8, 3

  ; exit t4
  mov rax, 60
  mov rdi, r8
  syscall
