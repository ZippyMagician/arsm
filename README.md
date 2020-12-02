# arsm
A toy version of Assembly, implemented in Rust.

## About
Mostly designed for fun, this flavour of Assembly was made for Rust specifically. See `/docs` for more a guide for using the language.

## Example Programs
Proof of concepts, some of these need to be implemented
### Hello, World!
```asm
str "Hello, World!" 0

.loop
  chr [ex]
  inc [ex]
  jmp db 0 ex :skip
  gto :loop
.skip
```
### Cat
```asm
.loop
  mov eax in
  jmp eax 3 :skip
  chr eax
  gto :loop
.skip
```
### Truth Machine
```asm
mov ex in
jmp ex 1 :loop
gto :skip
.loop
  out 1
  gto :loop
.skip
  out 0
```
