.loop
  mov eh in
  cz eh
  cjm :skip
  chr eh
  jmp :loop
..skip