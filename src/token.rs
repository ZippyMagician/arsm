#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Keyword(String),

    Numeric(i32),

    String(String),

    Branch(String),

    Register(String),

    Punctuation(char),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Op {
    // TODO: Implement binary operators
    BinOp(char, Box<Op>, Box<Op>),

    Cmd(String, Vec<Box<Op>>),

    Branch(String, Vec<Box<Op>>),

    Label(String),

    // TODO: Implement static memory referencing
    Memory(char, Box<Op>),

    Register(String),

    Numeric(i32),

    String(String),

    Empty,
}