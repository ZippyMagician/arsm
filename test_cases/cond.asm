.loop
    mov ax in
    cz ax
    chl 0

    ceq ax '0
    cmo ax '1
    chr ax
    jmp :loop