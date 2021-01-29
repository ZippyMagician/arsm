stk 10

.add
    psh 1 al
    inc al
    cmp al 10
    je :test
    jmp :add
..test
    out { stk.pop() }
    chr ',
    pop bl
    out bl
    sub al 2
    jz al :end
    chr ',
    jmp :test
..end