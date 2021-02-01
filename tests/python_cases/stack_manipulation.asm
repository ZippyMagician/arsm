stk 10

.add
    psh 1 al
    inc al
    ceq al 10
    cjm :test
    jmp :add
..test
    out { stk.pop() }
    chr ',
    pop bl
    out bl
    sub al 2
    cz al
    cjm :end
    chr ',
    jmp :test
..end