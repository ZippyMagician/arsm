use std::collections::HashMap;

macro_rules! insert_into {
    ($map:expr; $($name:literal $count:literal);*) => {
        $(
            $map.insert($name.to_string(), $count);
        )*
    };
}

// For `parser.rs`
pub const REGISTERS: &[char] = &['a', 'b', 'c', 'd', 'e', 'A', 'B', 'C', 'D', 'E'];
pub const REGISTER_ENDINGS: &[char] = &['x', 'X', 'h', 'H', 'l', 'L'];
pub const PUNCTUATION: &[&str] = &["+", "-", "*", "/", "[", "]", "#", "$", "@"];
lazy_static! {
    pub static ref COMMANDS: HashMap<String, usize> = {
        let mut m: HashMap<String, usize> = HashMap::new();
        insert_into!(m;
            "mov" 2;
            "inc" 1;
            "dec" 1;
            "out" 1;
            "jmp" 1;
            "je" 1;
            "jne" 1;
            "jl" 1;
            "jle" 1;
            "jg" 1;
            "jge" 1;
            "jz" 1;
            "mul" 2;
            "div" 2;
            "add" 2;
            "sub" 2;
            "str" 2;
            "db" 2;
            "in" 0;
            "chr" 1;
            "hlt" 1;
            "ret" 0;
            "cmp" 2;
            "stk" 1;
            "psh" 2;
            "pop" 1
        );
        m
    };
}

// For `mem.rs`
pub const U8_ALIGN: usize = std::mem::align_of::<u8>();
pub const REGISTRY_OFFSET: usize = 10;
// First 10 bytes are the registry, 11th byte for the cmp flags, next 9 are
// reserved for future use
pub const OFFSET: usize = REGISTRY_OFFSET + 10;
