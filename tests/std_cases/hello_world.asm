str "Hello, World!" 10

.loop
  chr #[eh]
  inc eh
  cmp db 0 10 eh
  je :skip
  jmp :loop
..skip