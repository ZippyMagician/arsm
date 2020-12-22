use std::iter::Peekable;

use clap::ArgMatches;
use num_traits::Num;

use crate::env::Environment;
use crate::registry::Position;
use crate::utils::{self, consts::*, status::Status, token::*};

fn flush(buf: &mut String, chr: char) {
    buf.clear();
    if !chr.is_whitespace() {
        buf.push(chr);
    }
}

fn is_num(test: &str) -> bool {
    test.parse::<i32>().is_ok()
}

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

pub fn construct_tree(stream: Vec<Node>) -> Vec<Op> {
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
                    let res = Box::new(current_tok(
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

pub fn parse(ast: Vec<Op>, matches: ArgMatches<'_>) {
    let mut v = match matches.value_of("stdin_file") {
        Some(path) => std::fs::read(path).unwrap_or_default(),
        None => matches
            .value_of("STDIN")
            .unwrap_or_default()
            .as_bytes()
            .into(),
    };

    let mut env = Environment::new(v.as_mut_slice());
    let mut ind = 0;

    while ind < ast.len() {
        if !run_op(&mut env, &ast, &mut ind).has_jmp() {
            ind += 1;
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
            let shallow_ref: Vec<&Op> = args.iter().collect();
            run_cmd(env, ast, ind, &*name, &shallow_ref)
        }

        Op::Branch(_, body) => {
            env.set_parent(&ast);
            for mut i in 0..body.len() {
                if run_op(env, &body, &mut i).has_jmp() {
                    env.clear_parent();
                    *ind = i;
                    return Box::new(true);
                }
            }
            env.clear_parent();
            Box::new(false)
        }

        _ => panic!("Invalid top-level op: {:?}", ast[*ind]),
    }
}

// Converts op to a numeric value
fn to_numeric<T: Num + num_traits::NumCast>(env: &mut Environment, ast: &[Op], obj: &Op) -> T {
    match obj {
        Op::Numeric(val) => num_traits::cast(*val),

        Op::Memory(ident, op) => {
            let val: usize = to_numeric(env, ast, &op);
            match ident {
                '#' => num_traits::cast(utils::read_from_mem_8(env.mem.as_mut(), val)),
                '$' => num_traits::cast(utils::read_from_mem_16(env.mem.as_mut(), val)),
                '@' => num_traits::cast(utils::read_from_mem_32(env.mem.as_mut(), val)),
                _ => panic!("Invalid memory identifier: '{}'", ident),
            }
        }

        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            if name.ends_with('x') {
                if name.len() == 3 {
                    num_traits::cast(env.registry.read_32(chrs[0], chrs[1]))
                } else {
                    num_traits::cast(env.registry.read_16(chrs[0]))
                }
            } else if name.ends_with('h') {
                num_traits::cast(env.registry.read_8(chrs[0], Position::Upper))
            } else {
                num_traits::cast(env.registry.read_8(chrs[0], Position::Lower))
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
                    let val = to_numeric(env, &ast, val);
                    env.registry.write_32(chrs[0], chrs[1], val);
                } else {
                    let val = to_numeric(env, &ast, val);
                    env.registry.write_16(chrs[0], val);
                }
            } else if name.ends_with('h') {
                let val = to_numeric(env, &ast, val);
                env.registry.write_8(chrs[0], Position::Upper, val);
            } else {
                let val = to_numeric(env, &ast, val);
                env.registry.write_8(chrs[0], Position::Lower, val);
            }
        }

        Op::Memory(ident, op) => {
            let pos: usize = to_numeric(env, &ast, &op);
            match ident {
                '#' => {
                    let val = to_numeric(env, &ast, val);
                    utils::write_to_mem_8(env.mem.as_mut(), pos, val)
                }

                '$' => {
                    let val = to_numeric(env, &ast, val);
                    utils::write_to_mem_16(env.mem.as_mut(), pos, val)
                }

                '@' => {
                    let val = to_numeric(env, &ast, val);
                    utils::write_to_mem_32(env.mem.as_mut(), pos, val)
                }
                _ => panic!("Invalid identifier for memory: '{}'", ident),
            }
        }
        _ => panic!("Invalid parameter: {:?}", obj),
    }
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
            Box::new(false)
        }

        "inc" => {
            // new_val is 1 more than the previous value
            let new_val = Box::new(Op::Numeric(1 + to_numeric::<i32>(env, ast, args[0])));
            modify_memory(env, ast, args[0], &new_val);
            Box::new(false)
        }

        "dec" => {
            // new_val is 1 less than the previous value
            let new_val = Box::new(Op::Numeric(to_numeric::<i32>(env, ast, args[0]) - 1));
            modify_memory(env, ast, args[0], &new_val);
            Box::new(false)
        }

        "out" => {
            print!("{}", to_numeric::<i32>(env, ast, args[0]));
            Box::new(false)
        }

        "chr" => {
            print!("{}", to_numeric::<u8>(env, ast, args[0]) as char);
            Box::new(false)
        }

        "jmp" => {
            *ind = to_numeric(env, ast, args[0]);
            Box::new(true)
        }

        "mul" => {
            // args[0] * args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &Box::new(Op::Numeric(left * right)));
            Box::new(false)
        }

        "div" => {
            // args[0] / args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &Box::new(Op::Numeric(left / right)));
            Box::new(false)
        }

        "sub" => {
            // args[0] - args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &Box::new(Op::Numeric(left - right)));
            Box::new(false)
        }

        "add" => {
            // args[0] + args[1] → args[0]
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            modify_memory(env, ast, args[0], &Box::new(Op::Numeric(left + right)));
            Box::new(false)
        }

        "je" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left == right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "jne" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left != right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "jz" => {
            let check: i32 = to_numeric(env, ast, args[0]);
            Box::new(if check == 0 {
                *ind = to_numeric(env, ast, args[1]);
                true
            } else {
                false
            })
        }

        "jg" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left > right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "jge" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left >= right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "jl" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left < right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "jle" => {
            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);
            Box::new(if left == right {
                *ind = to_numeric(env, ast, args[2]);
                true
            } else {
                false
            })
        }

        "str" => match args[0] {
            Op::String(val) => {
                for (i, chr) in val.chars().enumerate() {
                    utils::write_to_mem_8(env.mem.as_mut(), i, chr as u8);
                }
                let terminator: u8 = to_numeric(env, ast, args[1]);
                utils::write_to_mem_8(env.mem.as_mut(), val.len(), terminator);
                Box::new(false)
            }

            _ => panic!(
                "Argument #0 for command 'str' must be of type Op::String. Instead got: {:?}",
                ast[*ind]
            ),
        },

        "db" => {
            let mut i: usize = to_numeric(env, ast, args[0]);
            let terminator: u8 = to_numeric(env, ast, args[1]);
            let mut len = 0;
            while utils::read_from_mem_8(env.mem.as_mut(), i) != terminator {
                len += 1;
                i += 1;
            }
            Box::new(len)
        }

        "in" => Box::new(match env.stdin.next() {
            Some(val) => val as i32,
            None => 0,
        }),

        "hlt" => std::process::exit(to_numeric(env, ast, args[0])),

        _ => panic!("Command: {} unrecognized", cmd),
    }
}
