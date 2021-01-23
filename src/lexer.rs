use crate::utils::consts::{REGISTERS, REGISTER_ENDINGS, PUNCTUATION};
use crate::utils::token::Node;

#[inline]
fn flush(buf: &mut String, chr: char) {
    buf.clear();
    if !chr.is_whitespace() {
        buf.push(chr);
    }
}

#[inline]
fn is_num(test: &str) -> bool {
    test.parse::<i32>().is_ok()
}

#[inline]
fn is_register(test: &str) -> bool {
    test.ends_with(|chr| REGISTER_ENDINGS.contains(&chr))
        && test[..test.len() - 1]
            .chars()
            .all(|chr| REGISTERS.contains(&chr))
}

pub fn lex(program: &str) -> Vec<Node> {
    let mut prg = String::with_capacity(program.len() + 1);
    prg.push_str(program);
    prg.push('\n');
    let sep = prg.chars().collect::<Vec<char>>();

    let mut res = Vec::new();
    let mut buf = String::new();
    let mut i = 0;

    while i < sep.len() {
        let chr = sep[i];
        if is_num(&buf) {
            buf.push(chr);
            if !is_num(&buf) {
                buf.pop();
                res.push(Node::Numeric(buf.clone().parse::<i32>().unwrap()));
                flush(&mut buf, chr);
            }
        } else if buf.starts_with(':') || buf.starts_with('.') {
            if chr.is_whitespace() || !chr.is_alphabetic() {
                res.push(Node::Branch(buf.clone()));
                flush(&mut buf, chr);
            } else {
                buf.push(chr);
            }
        } else if PUNCTUATION.contains(&&*buf) {
            res.push(Node::Punctuation(
                *buf.chars().collect::<Vec<char>>().get(0).unwrap(),
            ));
            flush(&mut buf, chr);
        } else if buf == "\"" {
            buf.clear();
            while i < sep.len() && sep[i] != '"' {
                buf.push(sep[i]);
                i += 1;
            }
            res.push(Node::String(buf.clone()));
            buf.clear();
        } else if buf.starts_with('\'') {
            res.push(Node::Char(chr));
            buf.clear();
        } else if !chr.is_ascii_alphabetic() {
            if is_register(&buf) {
                res.push(Node::Register(buf.clone()));
            } else if !buf.is_empty() {
                res.push(Node::Keyword(buf.clone()));
            }

            flush(&mut buf, chr);
        } else {
            buf.push(chr);
        }

        i += 1;
    }

    res
}