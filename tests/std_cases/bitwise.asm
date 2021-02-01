mov ax 37
mov bx 68
mov cx 94
mov dx 3
mov ex 129

xor ax bx
ceq ax 97
cou 1

and bx cx
ceq bx 68
cou 1

or cx dx
ceq cx 95
cou 1

not ex
sub dx 133
ceq dx ex
cou 1