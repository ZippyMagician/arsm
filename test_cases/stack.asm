stk 50

.takeinp
    mov eh in
    cmp eh 0
    jz :outinp
    psh 1 eh
    inc ax
    jmp :takeinp
..outinp
    pop bl
    dec ax
    cmp ax 0
    chr bl
    jz :end
    jmp :outinp
..end