stk 10

sub ax 7
jmp :abs
out ax
chr ',

mov ax 13
jmp :abs
out ax
chr ',

mov ax 0
jmp :abs
out ax

hlt 0

.abs
    psh 2 ax
    mov ax { abs(fromBytes(stk[0:1])) }
    pop ex
    ret