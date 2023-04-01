BITS 64
DEFAULT REL

section .text
xor r13, r13
mov r13, 0
jmp0:
mov rax, 1
mov rdi, 1
lea rsi, str0
mov rdx, 17
syscall
inc r13
cmp r13, 10
jne 0x9
mov rax, 60
mov rdi, 69
syscall

section .rodata
str0: db "Hello World!",10,"UwU",10