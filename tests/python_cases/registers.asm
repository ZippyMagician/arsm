.loop
    mov al in
    mov ah in
    cz al
    chl 0
    cz ah
    chl 0

    out { @al + @ah }
    chr ',
    jmp :loop