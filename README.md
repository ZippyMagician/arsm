# arsm
A toy version of Assembly, implemented in Rust.

## About
Mostly designed for fun, this flavour of Assembly was made for Rust specifically. See `/etc/DOCS.md` for a guide to use the language.

## Installation
First, make sure you have [Rust](https://rust-lang.org) installed on your system. Then clone this repository into a local directory.
You have two options: installing normal arsm, or arsm with inline python support.
### Normal
run `cargo install --path path/to/repository`.
### Inline Python
Make sure you have [Python3](https://python.org) installed on your system. Then, run `cargo install --features inline-python --path path/to/repository`. Inline python can access the stack via the `stk` variable and registers via the `@` prefix, along with a few special functions
## Testing
Use the command `node test` to test every case in `./test_cases`. A new case called `N` can be created by:
 1. Creating `N.asm`, which is the program
 2. Creating `N.in`, which is the input for the program
 3. Creating `N.out`, which is the output the program should return
ARSM performs all numeric checks at runtime. ARSM will only compile in release mode when using the `inline-python` feature

## Example Programs
Some simple programs written in arsm
### Hello, World!
```asm
str "Hello, World!" 10

.loop
  chr #[eh]
  inc eh
  je db 0 10 eh :skip
  jmp :loop
..skip
```
### Cat
```asm
.loop
  mov eh in
  je eh 3 :skip
  chr eh
  jmp :loop
..skip
```
### Truth Machine
```asm
mov eh in
je eh 49 :loop
jmp :skip

.loop
  out 1
  jmp :loop
..skip
  out 0
```
