stk 50

.takeinp
    mov eh in
    jz eh :outinp
    psh 1 eh
    inc ax
    jmp :takeinp
..outinp
    pop bl
    dec ax
    chr bl
    jz ax :end
    jmp :outinp
..end