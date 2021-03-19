Test version of arsm using a union instead of traits.

# Information
`std_cases/hello_world.asm` -> ~20 microseconds less than using `num_traits`
`std_cases/stack.asm` -> Much faster for larger inputs, implies that the more complex the program, the wider the gap between these two versions

Uses some strange `union` behaviour, which has lead to only `--release` causing the project to run properly, and may lead to some issues in the future.

I honestly am not suprised using a `union` and a lot of `unsafe`s is faster than a `num_trait` extension with lots of methods, as the latter required a lot of `Box`es and `dyn`. For now I'll leave the two branches unmerged, as it is apparent this version has some serious issues with it not working sometimes.