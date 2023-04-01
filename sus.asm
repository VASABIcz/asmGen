BITS 64
DEFAULT REL

section .text
xor r11, r11
mov r11, 0
jmp0:
mov rax, 1
mov rdi, 1
lea rsi, str0
mov rdx, 17
syscall
inc r11
cmp r11, 10
jne jmp0
mov rax, 60
mov rdi, 420
syscall

section .rodata
str0: db "Hello World!",10,"UwU",10
