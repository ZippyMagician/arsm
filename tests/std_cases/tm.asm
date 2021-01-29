mov eh in
cmp eh 49
je :loop
jmp :skip

.loop
  out 1
  jmp :loop
..skip
  out 0