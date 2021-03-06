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
### Memory
![The memory layout](https://raw.githubusercontent.com/ZippyMagician/arsm/master/etc/arsm_memory.png)
#### Registry
There are 5 16-bit registries: *a*, *b*, *c*, *d*, and *e*. To call a single registry, attach an *x* onto the end. To join two registries together (making 32-bits of data), simply put two separate registry names before the *x*. To only access half of the 16-bit registries, use *h* or *l* (upper or lower respectively).
#### Cells
Memory is accessed by placing some expression that evaluates down to a number inside `[]`. Memory is seperated into 8-bit cells, which similar to the registry can be joined together. To accomplish this, you prefix the `[]` with an identifier. *#* marks a single cell (8-bits), *$* marks two cells (16-bits), and *@* marks four cells (32-bits).
### Characters
A character literal is denoted by a `'` followed by any ascii character. This will yield the integer value of that character, for use in the program.
## Commands
* `mov <A> <B>` -> Move B into A
* `inc <A>` -> Increment A
* `dec <A>` -> Decrement A
* `out <A>` -> Print A with no newline
* `jmp <A>` -> Goto the branch entitled A (note: A is a **label**, not a **branch**)
* `mul <A> <B>` -> Multiply A by B, store in A
* `div <A> <B>` -> Divide A by B, store in A
* `add <A> <B>` -> Add A and B, store in A
* `sub <A> <B>` -> Subtract B from A, store in A
* `lsh <A> <B>` -> A << B, store in A
* `rsh <A> <B>` -> A >> B, store in A
* `or <A> <B>` -> A | B, store in A
* `xor <A> <B>` -> A ^ B, store in A
* `and <A> <B>` -> A & B, store in A
* `not <A>` -> ~A
* `str <A> <B>` -> Place string A in memory, with final character B
* `db <A> <B>` -> Get length of data, starting at point A in memory and ending when the point in memory equals B
* `in` -> Get next byte of STDIN or a null-byte (`0`) if none left
* `chr <A>` -> Print A as a character instead of number
* `hlt <A>` -> Terminates program with exit code A
* `ret` -> Return to the point at which this jump was called from
* `stk <A>` -> Resizes stack to size A. Defaults to 0
* `psh <A> <B>` -> Pushes A byte number B to stack
* `pop <A>` -> Pops N bytes (enough to fill A) and move to A.
* `swp <A> <B>` -> Swaps A and B's values
* `ceq <A> <B>` -> Sets conditional flag if A == B
* `cz <A>` -> Sets conditional flag if A is 0
* `cne <A> <B>` -> Sets conditional flag if A != B
* `cg <A> <B>` -> Sets conditional flag if A > B
* `cge <A> <B>` -> Sets conditional flag if A >= B
* `cl <A> <B>` -> Sets conditional flag if A < B
* `cle <A> <B>` -> Sets conditional flag if A <= B

Additionally, there are conditional versions of the following:
jmp, mov, inc, dec, out, mul, div, add, sub, lsh, rsh, or, and, xor, not, chr, hlt, ret, psh, pop, swp (remove the last letter, put a `c` in the front)
## Inline Python
Inline Python supports a few custom functions + variables to manipulate and make use of

| Name | Type | Description |
| ---- | ---- | ----------- |
| `fromBytes` | func | Converts list $1 `u8` bytes to single, signed integer. Useful for joining sections of the stack |
| `stk` | var | The current stack stored in memory. Can be modified. WARNING: EXCEEDING THE BOUNDS OF THE STACK **WILL NOT** ERROR, AND MAY LEAD TO SOME DATA BEING LOST |
| `popN` | func | Pops the top $2 items from the list $1 |

In addition, you can access any register (for example `eax`) by prefixing it with an `@`. For instance:
```asm
mov ex 14
mov ah 56

ceq { @ex + @ah } 70
chl 1
```