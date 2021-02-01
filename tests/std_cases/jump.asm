.loop
    jmp :helper
    chr eh
    jmp :loop
..helper
    mov eh in
    cz eh
    cjm :end
    ret
..end
    hlt 0