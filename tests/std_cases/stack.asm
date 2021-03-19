stk 100

.takeinp
    mov eh in
    cz eh
    cjm :outinp
    psh 1 eh
    inc ax
    jmp :takeinp
..outinp
    pop bl
    dec ax
    chr bl
    cz ax
    cjm :end
    jmp :outinp
..end