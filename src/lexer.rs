use crate::utils::consts::{PUNCTUATION, REGISTERS, REGISTER_ENDINGS};
use crate::utils::token::Node;

#[inline]
fn flush(buf: &mut String, chr: char) {
    buf.clear();
    if !chr.is_whitespace() {
        buf.push(chr);
    }
}

pub fn lex(program: &str) -> Vec<Node> {
    let mut prg = program.to_string();
    prg.push('\n');
    let sep = prg.chars().collect::<Vec<char>>();

    // The number of spaces will give a rough estimate of how large the returned `Vec` will be, improving performance
    let mut res = Vec::with_capacity(program.matches(' ').count());
    let mut buf = String::new();
    let mut i = 0;

    while i < sep.len() {
        let chr = sep[i];
        if buf.parse::<i32>().is_ok() {
            buf.push(chr);
            if buf.parse::<i32>().is_err() {
                buf.pop();
                res.push(Node::Numeric(buf.parse::<i32>().unwrap()));

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
            res.push(Node::Punctuation(buf.chars().next().unwrap()));

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
            if buf.ends_with(|chr| REGISTER_ENDINGS.contains(&chr))
                && buf[..buf.len() - 1]
                    .chars()
                    .all(|chr| REGISTERS.contains(&chr))
            {
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

#[cfg(test)]
mod lex_tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(lex("").is_empty());
    }

    #[test]
    fn test_full() {
        assert_eq!(
            lex("mov eh abx 13 @ +"),
            vec![
                Node::Keyword("mov".to_string()),
                Node::Register("eh".to_string()),
                Node::Register("abx".to_string()),
                Node::Numeric(13),
                Node::Punctuation('@'),
                Node::Punctuation('+')
            ]
        );
    }
}
