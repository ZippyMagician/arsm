.main
    str "This is an error" 10
    cmp in 0
    jz :panic
    jmp :skipanic
.

.panic
    mov eh 0
    jmp :panicloop
..panicloop
  chr #[eh]
  inc eh
  cmp db 0 10 eh
  je :skipanic
  jmp :panicloop
..skipanic
    hlt 0

chr 78