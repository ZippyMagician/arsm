.loop
    jmp :helper
    chr eh
    jmp :loop
..helper
    mov eh in
    je eh 0 :end
    ret
..end
    hlt 0