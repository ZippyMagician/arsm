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
.end
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
Memory is accessed by placing some expression that evaluates down to a number inside `[]`. Memory is 1024 slots of 8-bits, but similarly to registries can be joined together. To accomplish this, you prefix the `[]` with an identifier. *B* marks a single cell (8-bits), *W* marks two cells (16-bits), and *DW* marks four cells (32-bits).

## Commands
* `mov <A> <B>` -> Move B into A
* `inc <A>` -> Increment A
* `dec <A>` -> Decrement A
* `out <A>` -> Print A with no newline
* `goto <A>` -> Goto the branch entitled A (note: A is a **label**, not a **branch**)
* `mul <A> <B>` -> Multiply A by B, store in A
* `div <A> <B>` -> Divide A by B, store in A
* `add <A> <B>` -> Add A and B, store in A
* `sub <A> <B>` -> Subtract B from A, store in A