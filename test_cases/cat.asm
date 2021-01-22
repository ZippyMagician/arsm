.loop
  mov eh in
  cmp eh 0
  jz :skip
  chr eh
  jmp :loop
..skip