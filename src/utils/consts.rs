use std::collections::HashMap;

#[cfg(feature = "inline-python")]
use regex::Regex;

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
            "mov" 2; "cmo" 2; "inc" 1; "cin" 1; "dec" 1;
            "cde" 1; "out" 1; "cou" 1; "jmp" 1; "cjm" 1;
            "mul" 2; "div" 2; "add" 2; "sub" 2; "cmu" 2;
            "cdi" 2; "cad" 2; "csu" 2; "str" 2; "db"  2;
            "in"  0; "chr" 1; "cch" 1; "hlt" 1; "chl" 1;
            "ret" 0; "cre" 0; "cmp" 2; "stk" 1; "psh" 2;
            "cps" 2; "pop" 1; "cpo" 1; "ceq" 2; "cg"  2;
            "cge" 2; "cl"  2; "cle" 2; "cne" 2; "cz"  1;
            "rsh" 2; "crs" 2; "lsh" 2; "cls" 2; "or"  2;
            "cor" 2; "xor" 2; "cxo" 2; "and" 2; "can" 2;
            "not" 1; "cno" 1; "swp" 2; "csw" 2
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

// Python backend. Used by `utils/mod.rs`
#[cfg(feature = "inline-python")]
pub const PYTHON_HEAD: &'static str = r#"
fromBytes = lambda n: int.from_bytes(n, signed=True, byteorder=__import__('sys').byteorder)
popN = lambda n, count: [n.pop() for i in range(count)][::-1] 

ret = eval"#;

#[cfg(feature = "inline-python")]
lazy_static! {
    pub static ref REGISTER_REGEX: Regex = Regex::new(r#"@([a-e][hl]|[a-e]{1,2}x)"#).unwrap();
}
