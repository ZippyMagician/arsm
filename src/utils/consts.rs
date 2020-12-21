use std::collections::HashMap;

macro_rules! insert_into {
    ($map:expr; $($name:literal $count:literal);*) => {
        $(
            $map.insert($name.to_string(), $count);
        )*
    };
}

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
            "je" 3;
            "jne" 3;
            "jl" 3;
            "jle" 3;
            "jg" 3;
            "jge" 3;
            "jz" 2;
            "mul" 2;
            "div" 2;
            "add" 2;
            "sub" 2;
            "str" 2;
            "db" 2;
            "in" 0;
            "chr" 1;
            "hlt" 1
        );
        m
    };
}
