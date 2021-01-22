.loop
    jmp :helper
    chr eh
    jmp :loop
..helper
    mov eh in
    cmp eh 0
    jz :end
    ret
..end
    hlt 0