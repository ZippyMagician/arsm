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
    let mut sep = prg.chars();

    // The number of spaces will give a rough estimate of how large the returned
    // `Vec` will be, improving performance
    let mut res = Vec::with_capacity(program.matches(' ').count());
    let mut buf = String::new();

    while let Some(chr) = sep.next() {
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
            buf.push(chr);
            // An alternative would be `buf.push_str(sep.take_while(|&a| a != '"').collect::<String>())`,
            // but that leads to errors with mutable borrowing. This is the next best thing. Also,
            // turns out clippy errors here even though using `for str_chr in sep` won't work
            #[allow(clippy::while_let_on_iterator)]
            while let Some(str_chr) = sep.next() {
                if str_chr == '"' {
                    break;
                }

                buf.push(str_chr);
            }

            res.push(Node::String(buf.clone()));
            buf.clear();
        } else if buf == "{" {
            #[cfg(not(feature = "inline-python"))]
            panic!("Cannot use inline python code when the feature is disabled");

            #[cfg(feature = "inline-python")]
            {
                buf.clear();
                buf.push(chr);

                // Same as above
                #[allow(clippy::clippy::while_let_on_iterator)]
                while let Some(py_chr) = sep.next() {
                    if py_chr == '}' {
                        break;
                    }

                    buf.push(py_chr);
                }

                res.push(Node::InlinePy(buf.clone()));
                buf.clear();
            }
        } else if buf.starts_with('\'') {
            res.push(Node::Char(chr));
            buf.clear();
        } else if !chr.is_ascii_alphabetic() {
            if buf.ends_with(REGISTER_ENDINGS)
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

    #[test]
    fn test_strings() {
        assert_eq!(
            lex("\"Hello\" eh + 4"),
            vec![
                Node::String("Hello".to_string()),
                Node::Register("eh".to_string()),
                Node::Punctuation('+'),
                Node::Numeric(4)
            ]
        );
    }
}
