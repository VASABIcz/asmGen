BITS 64
DEFAULT REL

section .text
mov rax, 1
mov rdi, 1
lea rsi, str3838560998012430438
mov rdx, 13
syscall
ret

section .rodata
str3838560998012430438: db "Hello World!\n"