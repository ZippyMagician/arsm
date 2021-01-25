use std::iter::Peekable;

use crate::bx;
use crate::utils::{consts::COMMANDS, token::*};

pub fn construct_tree(stream: &[Node]) -> Vec<Op> {
    let mut res = Vec::new();
    // Note: using a peekable iterator isn't really necessary yet, but it will be once I implement Node::Punctuation
    let mut stream = stream.iter().peekable();

    while let Some(tok) = stream.next() {
        let op = current_tok(&mut stream, tok);
        if op != Op::Empty {
            res.push(op);
        }
    }

    res
}

pub fn current_tok<'a>(stream: &mut Peekable<impl Iterator<Item = &'a Node>>, cur: &Node) -> Op {
    match *cur {
        Node::Keyword(ref name) => {
            if let Some(&count) = COMMANDS.get(name) {
                let mut v = Vec::with_capacity(count);

                while v.len() < count {
                    if let Some(n) = stream.next() {
                        let t = current_tok(stream, n);
                        if t != Op::Empty {
                            v.push(t);
                        }
                    } else {
                        panic!(
                            "{} takes {} arguments but {} were provided",
                            name,
                            count,
                            v.len()
                        );
                    }
                }

                Op::Cmd(name.clone(), v)
            } else {
                panic!("Unrecognized command: {}", name);
            }
        }

        // TODO: Simple math parser
        // Math: eax + 3 * ah
        // Memory: B[ah + 1], W[ah], DW[eax * 3 + 1]
        Node::Punctuation(ref chr) => {
            if *chr == '#' || *chr == '$' || *chr == '@' {
                if let Some(Node::Punctuation('[')) = stream.next() {
                    let tok = stream.next();
                    let res = bx!(current_tok(
                        stream,
                        tok.expect("Invalid termination of a memory identifier: Missing body"),
                    ));

                    if let Some(Node::Punctuation(']')) = stream.next() {
                        Op::Memory(*chr, res)
                    } else {
                        panic!("Invalid termination of a memory identifier: Missing ']'")
                    }
                } else {
                    panic!("Invalid beginning to a memory identifier: Missing '['")
                }
            } else if *chr == '[' || *chr == ']' {
                panic!("Invalid free-standing punctuation '{}'.", chr);
            } else {
                todo!("Math unimplemented");
            }
        }

        Node::Numeric(val) => Op::Numeric(val),

        Node::String(ref str) => Op::String(str.clone()),

        Node::Branch(ref name) => {
            let name = name.clone();

            if name.starts_with(':') {
                Op::Label(name)
            } else {
                let mut v = Vec::new();
                while let Some(node) = stream.next() {
                    if Node::Branch(String::from('.')) == *node {
                        break;
                    }

                    v.push(current_tok(stream, node));
                }

                Op::Branch(name, v)
            }
        }

        Node::Register(ref name) => Op::Register(name.clone()),

        Node::Char(ref chr) => Op::Char(*chr),

        #[cfg(feature = "inline-python")]
        Node::InlinePy(ref val) => Op::InlinePy(val.clone()),
    }
}
