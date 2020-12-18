# Documentation
What follows is the documentation and general structure of the arsm programming language. This is an all-encompassing, explanatory guide that will always be up to date.

## General Structure
### Labels
Labels are used when you want to pass a branch as an argument. They are defined as:
```
:name
```
### Branches
Branches can be jumped to by certain commands. They are defined as follows:
```
.name
    ...
.
```
A branch will be executed on the programs first run-through. For example:
```
.main
    mov eax 13
    out eax
```
Will print 13, even though no specific call to `.main` was made
### Registries
There are 5 16-bit registries: *a*, *b*, *c*, *d*, and *e*. To call a single registry, attach an *x* onto the end. To join two registries together (making 32-bits of data), simply put two separate registry names before the *x*. To only access half of the 16-bit registries, use *h* or *l* (upper or lower respectively).
### Memory
Memory is accessed by placing some expression that evaluates down to a number inside `[]`. Memory is 1024 slots of 8-bits, but similarly to registries can be joined together. To accomplish this, you prefix the `[]` with an identifier. *#* marks a single cell (8-bits), *$* marks two cells (16-bits), and *@* marks four cells (32-bits).
### Characters
A character literal is denoted by a `'` followed by any ascii character. This will yield the integer value of that character, for use in the program.
## Commands
* `mov <A> <B>` -> Move B into A
* `inc <A>` -> Increment A
* `dec <A>` -> Decrement A
* `out <A>` -> Print A with no newline
* `jmp <A>` -> Goto the branch entitled A (note: A is a **label**, not a **branch**)
* `je <A> <B> <C>` -> Goto branch C if A == B
* `jne <A> <B> <C>` -> Goto branch C if A != B
* `jz <A> <B>` -> Goto branch B if A == 0
* `jg <A> <B> <C>` -> Goto branch C if A > B
* `jge <A> <B> <C>` -> Goto branch C if A >= B
* `jl <A> <B> <C>` -> Goto branch C if A < B
* `jle <A> <B> <C>` -> Goto branch C if A <= B
* `mul <A> <B>` -> Multiply A by B, store in A
* `div <A> <B>` -> Divide A by B, store in A
* `add <A> <B>` -> Add A and B, store in A
* `sub <A> <B>` -> Subtract B from A, store in A
* `str <A> <B>` -> Place string A in memory, with final character B
* `db <A> <B>` -> Get length of data, starting at point A in memory and ending when the point in memory equals B
* `in` -> Get next byte of STDIN or a null-byte (`0`) if none left
* `chr <A>` -> Print A as a character instead of number
