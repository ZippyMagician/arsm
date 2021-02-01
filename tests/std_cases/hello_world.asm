str "Hello, World!" 10

.loop
  chr #[eh]
  inc eh
  ceq db 0 10 eh
  cjm :skip
  jmp :loop
..skip