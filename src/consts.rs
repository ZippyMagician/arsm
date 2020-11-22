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
pub const PUNCTUATION: &[&str] = &["+", "-", "*", "/", "[", "]"];

lazy_static! {
    pub static ref COMMANDS: HashMap<String, usize> = {
        let mut m: HashMap<String, usize> = HashMap::new();
        insert_into!(m;
            "mov" 2;
            "inc" 1;
            "dec" 1;
            "out" 1;
            "jmp" 3;
            "goto" 1;
            "mul" 2;
            "div" 2;
            "add" 2;
            "sub" 2
        );
        m
    };
}
