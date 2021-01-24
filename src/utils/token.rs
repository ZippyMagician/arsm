#[derive(Debug, PartialEq, Clone)]
pub enum Node {
	Keyword(String),

	Numeric(i32),

	String(String),

	Branch(String),

	Register(String),

	Punctuation(char),

	Char(char),
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

	Empty,
}
