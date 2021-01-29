.loop
    jmp :helper
    chr eh
    jmp :loop
..helper
    mov eh in
    jz eh :end
    ret
..end
    hlt 0