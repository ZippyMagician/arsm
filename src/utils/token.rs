#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Keyword(String),

    Numeric(i32),

    String(String),

    Branch(String),

    Register(String),

    Punctuation(char),

    Char(char),

    #[cfg(feature = "inline-python")]
    InlinePy(String),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Op {
    // TODO: Implement binary operators
    BinOp(char, Box<Op>, Box<Op>),

    Cmd(String, Vec<Op>),

    Branch(String, Vec<Op>),

    Label(String),

    Memory(char, Box<Op>),

    Register(String),

    Numeric(i32),

    String(String),

    Char(char),

    #[cfg(feature = "inline-python")]
    InlinePy(String),

    // Emtpy stack marker to tell `modify_memory` that it needs to pop from the stack
    StackMarker,

    Empty,
}
