# arsm
A toy version of Assembly, implemented in Rust.

## About
Mostly designed for fun, this flavour of Assembly was made for Rust specifically. See `/docs` for more a guide for using the language.

## Example Programs
Proof of concepts, some of these need to be implemented
### Hello, World!
```asm
str "Hello, World!" 10

.loop
  chr [eh]
  inc eh
  jmp db 0 10 eh :skip
  gto :loop
.skip
```
### Cat
```asm
.loop
  mov eh in
  jmp eh 3 :skip
  chr eh
  gto :loop
.skip
```
### Truth Machine
```asm
mov eh in
jmp eh 49 :loop
gto :skip
.loop
  out 1
  gto :loop
.skip
  out 0
```
