use std::iter::Peekable;

use clap::ArgMatches;

use crate::bx;
use crate::env::Environment;
use crate::utils::{consts::*, token::*, traits::*};

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

pub fn current_tok(stream: &mut Peekable<std::slice::Iter<'_, Node>>, cur: &Node) -> Op {
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
                        tok.unwrap_or_else(|| {
                            panic!("Invalid termination of a memory identifier: Missing body")
                        }),
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

        Node::Numeric(ref val) => Op::Numeric(*val),

        Node::String(ref str) => Op::String(str.clone()),

        Node::Branch(ref name) => {
            if name.starts_with(':') {
                Op::Label(name.clone())
            } else {
                let mut v = Vec::new();
                while let Some(node) = stream.next() {
                    if Node::Branch(".".to_string()) == *node {
                        break;
                    }

                    v.push(current_tok(stream, node));
                }

                Op::Branch(name.clone(), v)
            }
        }

        Node::Register(ref name) => Op::Register(name.clone()),

        Node::Char(ref chr) => Op::Char(*chr),
    }
}

pub fn parse(ast: &[Op], matches: &ArgMatches<'_>) {
    let mut v = if matches.is_present("file") {
        std::fs::read(matches.value_of("STDIN").unwrap_or_default()).unwrap_or_default()
    } else if matches.is_present("user") {
        matches
            .value_of("STDIN")
            .unwrap_or_default()
            .as_bytes()
            .into()
    } else {
        Vec::new()
    };

    let mut env = Environment::new(v.as_mut_slice());
    env.set_parent(ast);
    let mut ind = 0;

    while ind < ast.len() {
        if !run_op(&mut env, &ast, &mut ind).has_jmp() {
            ind += 1;
            env.pos.0 += 1;
        }
    }

    if matches.is_present("debug") {
        println!("\nDump: {:?}", env);
    }
}

// Returns true if the index was manually updated
fn run_op(env: &mut Environment, ast: &[Op], ind: &mut usize) -> Box<dyn Status> {
    match &ast[*ind] {
        Op::Cmd(name, args) => {
            if env.jump_point.len() > 0 {
                env.jump_point.clear();
            }

            let shallow_ref: Vec<&Op> = args.iter().collect();
            run_cmd(env, ast, ind, &*name, &shallow_ref)
        }

        Op::Branch(_, body) => {
            for mut i in env.pos.1..body.len() {
                if run_op(env, &body, &mut i).has_jmp() {
                    *ind = i;
                    return bx!(true);
                } else {
                    env.pos.1 += 1;
                }
            }
            
            bx!(false)
        }

        _ => panic!("Invalid top-level op: {:?}", ast[*ind]),
    }
}

// Converts op to a numeric value
fn to_numeric<T: Num>(env: &mut Environment, ast: &[Op], obj: &Op) -> T {
    match obj {
        Op::Numeric(val) => num_traits::cast(*val),

        Op::Memory(ident, op) => {
            let val = to_numeric(env, ast, &op);

            match ident {
                '#' => num_traits::cast(env.mem.m_read::<u8>(val)),
                '$' => num_traits::cast(env.mem.m_read::<i16>(val)),
                '@' => num_traits::cast(env.mem.m_read::<i32>(val)),
                _ => panic!("Invalid memory identifier: '{}'", ident),
            }
        }

        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            if name.ends_with('x') {
                if name.len() == 3 {
                    num_traits::cast(env.mem.r_read::<i32>(&(chrs[0], chrs[1])))
                } else {
                    num_traits::cast(env.mem.r_read::<i16>(&chrs[0]))
                }
            } else if name.ends_with('h') {
                num_traits::cast(env.mem.r_read::<u8>(&(chrs[0], Pos::Upper)))
            } else {
                num_traits::cast(env.mem.r_read::<u8>(&(chrs[0], Pos::Lower)))
            }
        }

        Op::Label(name) => num_traits::cast(
            env.get_parent()
                .clone()
                .unwrap_or_else(|| ast.to_owned())
                .iter()
                .position(|entry| {
                    if let Op::Branch(n, _) = entry {
                        n[1..] == name[1..]
                    } else {
                        false
                    }
                })
                .unwrap(),
        ),

        Op::BinOp(_, _, _) => todo!("Math unimplemented"),

        Op::Char(chr) => num_traits::cast(*chr as u8),

        Op::Cmd(name, args) => {
            let mut dummy_ind = 0;
            let args: Vec<&Op> = args.iter().collect();
            num_traits::cast(run_cmd(env, ast, &mut dummy_ind, &*name, args.as_slice()).get_val())
        }

        _ => panic!("Invalid numeric literal: {:?}", obj),
    }
    .unwrap_or_else(|| {
        panic!(
            "Could not convert {:?} to type <{}>",
            obj,
            std::any::type_name::<T>()
        )
    })
}

// Pass in the op in which memory is modified, and it will automatically update it with the value
fn modify_memory(env: &mut Environment, ast: &[Op], obj: &Op, val: &Op) {
    match obj {
        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            if name.ends_with('x') {
                if chrs.len() == 3 {
                    let val: i32 = to_numeric(env, &ast, val);
                    env.mem.r_write(&(chrs[0], chrs[1]), &val);
                } else {
                    let val: i16 = to_numeric(env, &ast, val);
                    env.mem.r_write(&chrs[0], &val);
                }
            } else if name.ends_with('h') {
                let val: u8 = to_numeric(env, &ast, val);
                env.mem.r_write(&(chrs[0], Pos::Upper), &val);
            } else {
                let val: u8 = to_numeric(env, &ast, val);
                env.mem.r_write(&(chrs[0], Pos::Lower), &val);
            }
        }

        Op::Memory(ident, op) => {
            let pos = to_numeric(env, &ast, &op);
            match ident {
                '#' => {
                    let val: u8 = to_numeric(env, &ast, val);
                    env.mem.m_write(pos, &val);
                }

                '$' => {
                    let val: i16 = to_numeric(env, &ast, val);
                    env.mem.m_write(pos, &val);
                }

                '@' => {
                    let val: i32 = to_numeric(env, &ast, val);
                    env.mem.m_write(pos, &val);
                }

                _ => panic!("Invalid identifier for memory: '{}'", ident),
            }
        }
        _ => panic!("Invalid parameter: {:?}", obj),
    }
}

macro_rules! set_ind {
    ($ind:ident, $env:ident, $val:expr) => {
        *$ind = $val;
        $env.jump_point.push($env.pos);
        $env.pos = (*$ind, 0);
    };
}

// Returns `true` if `ind` was modified, `false` otherwise
fn run_cmd(
    env: &mut Environment,
    ast: &[Op],
    ind: &mut usize,
    cmd: &str,
    args: &[&Op],
) -> Box<dyn Status> {
    match cmd {
        "mov" => {
            // Move second value into the first
            modify_memory(env, ast, args[0], args[1]);
            bx!(false)
        }

        "inc" => {
            // new_val is 1 more than the previous value
            let new_val = bx!(Op::Numeric(1 + to_numeric::<i32>(env, ast, args[0])));
            modify_memory(env, ast, args[0], &new_val);
            bx!(false)
        }

        "dec" => {
            // new_val is 1 less than the previous value
            let new_val = bx!(Op::Numeric(to_numeric::<i32>(env, ast, args[0]) - 1));
            modify_memory(env, ast, args[0], &new_val);
            bx!(false)
        }

        "out" => {
            print!("{}", to_numeric::<i32>(env, ast, args[0]));
            bx!(false)
        }

        "chr" => {
            print!("{}", to_numeric::<u8>(env, ast, args[0]) as char);
            bx!(false)
        }

        "jmp" => {
            set_ind!(ind, env, to_numeric(env, ast, args[0]));
            bx!(true)
        }

        "mul" => {
            // args[0] * args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &bx!(Op::Numeric(left * right)));
            bx!(false)
        }

        "div" => {
            // args[0] / args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &bx!(Op::Numeric(left / right)));
            bx!(false)
        }

        "sub" => {
            // args[0] - args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &bx!(Op::Numeric(left - right)));
            bx!(false)
        }

        "add" => {
            // args[0] + args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &bx!(Op::Numeric(left + right)));
            bx!(false)
        }

        "je" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left == right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jne" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left != right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jz" => {
            let check: i32 = to_numeric(env, ast, args[0]);
            bx!(if check == 0 {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jg" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left > right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jge" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left >= right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jl" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left < right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "jle" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            bx!(if left <= right {
                set_ind!(ind, env, to_numeric(env, ast, args[2]));
                true
            } else {
                false
            })
        }

        "str" => match args[0] {
            Op::String(val) => {
                for (i, chr) in val.chars().enumerate() {
                    env.mem.m_write(i, &(chr as u8));
                }

                let terminator: u8 = to_numeric(env, ast, args[1]);
                env.mem.m_write(val.len(), &terminator);
                bx!(false)
            }

            _ => panic!(
                "Argument #0 for command 'str' must be of type Op::String. Instead got: {:?}",
                ast[*ind]
            ),
        },

        "db" => {
            let mut i = to_numeric(env, ast, args[0]);
            let terminator: u8 = to_numeric(env, ast, args[1]);
            let mut len = 0;
            while env.mem.m_read::<u8>(i) != terminator {
                len += 1;
                i += 1;
            }
            bx!(len)
        }

        "in" => bx!(match env.stdin.next() {
            Some(val) => i32::from(val),
            None => 0,
        }),

        "ret" => {
            if env.jump_point.len() > 0 {
                let (left, right) = env.jump_point.pop().unwrap();
                env.pos = (left, right + 1);
                *ind = left;
            } else {
                panic!("Cannot return");
            }

            bx!(true)
        }

        "hlt" => std::process::exit(to_numeric(env, ast, args[0])),

        _ => panic!("Command: {} unrecognized", cmd),
    }
}
