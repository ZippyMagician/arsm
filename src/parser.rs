use clap::ArgMatches;

use crate::token::*;
// use crate::utils;
use crate::consts::*;
use crate::env::Environment;
use crate::registry::Position;

fn flush(buf: &mut String, chr: char) {
    *buf = String::new();
    if !chr.is_whitespace() {
        buf.push(chr);
    }
}

fn is_num(test: &String) -> bool {
    test.parse::<i32>().is_ok()
}

fn is_register(test: &String) -> bool {
    test.ends_with(|chr| REGISTER_ENDINGS.contains(&chr))
        && test[..test.len() - 1]
            .chars()
            .all(|chr| REGISTERS.contains(&chr))
}

pub fn lex(program: &str) -> Vec<Node> {
    let mut prg = String::with_capacity(program.len() + 1);
    prg.push_str(program);
    prg.push_str("\n");
    let sep = prg.chars().collect::<Vec<char>>();

    let mut res = Vec::new();
    let mut buf = String::new();

    for mut i in 0..sep.len() {
        let chr = sep[i];
        if is_num(&buf) {
            buf.push(chr);
            if !is_num(&buf) {
                buf.pop();
                res.push(Node::Numeric(buf.clone().parse::<i32>().unwrap()));
                flush(&mut buf, chr);
            }
        } else if buf.starts_with(':') || buf.starts_with('.') {
            if chr.is_whitespace() {
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
            i += 1;
            while sep[i] != '"' {
                buf.push(sep[i]);
                i += 1;
            }
            res.push(Node::String(buf.clone()[1..buf.len() - 1].to_string()));
            flush(&mut buf, sep[i]);
        } else {
            if chr.is_whitespace() {
                if is_register(&buf) {
                    res.push(Node::Register(buf.clone()));
                } else if buf != "" {
                    res.push(Node::Keyword(buf.clone()));
                }

                flush(&mut buf, chr);
            } else {
                buf.push(chr);
            }
        }
    }

    res
}

pub fn construct_tree(stream: Vec<Node>) -> Vec<Box<Op>> {
    let mut res = Vec::new();
    // Note: using a peekable iterator isn't really necessary yet, but it will be once I implement Node::Punctuation
    let mut stream = stream.iter().peekable();

    while let Some(tok) = stream.next() {
        res.push(Box::new(current_tok(&mut stream, tok)));
    }

    res
}

pub fn current_tok(stream: &mut std::iter::Peekable<std::slice::Iter<'_, Node>>, cur: &Node) -> Op {
    match *cur {
        Node::Keyword(ref name) => {
            if let Some(&count) = COMMANDS.get(name) {
                let mut v = Vec::with_capacity(count);
                for i in 0..count {
                    if let Some(n) = stream.next() {
                        v.push(Box::new(current_tok(stream, n)));
                    } else {
                        panic!("{} takes {} arguments but {} were provided", name, count, i);
                    }
                }
                Op::Cmd(name.clone(), v)
            } else {
                panic!("Unrecognized command: {}", name);
            }
        }

        // TODO: Simple math parser and memory parser
        // Math: eax + 3 * ah
        // Memory: B[ah + 1], W[ah], DW[eax * 3 + 1]
        Node::Punctuation(_) => panic!("Punctuation unimplemented"),

        Node::Numeric(ref val) => Op::Numeric(val.clone()),

        Node::String(ref str) => Op::String(str.clone()),

        Node::Branch(ref name) => {
            if name.starts_with(':') {
                Op::Label(name.clone())
            } else {
                let mut v = Vec::new();
                while let Some(node) = stream.next() {
                    if Node::Branch(".end".to_string()) == *node {
                        break;
                    }

                    v.push(Box::new(current_tok(stream, node)));
                }

                Op::Branch(name.clone(), v)
            }
        }

        Node::Register(ref name) => Op::Register(name.clone()),
    }
}

pub fn parse(ast: Vec<Box<Op>>, matches: ArgMatches) {
    let mut env = Environment::new();
    let mut ind = 0;

    while ind < ast.len() {
        if !run_op(&mut env, &ast, &mut ind) {
            ind += 1;
        }
    }

    if matches.is_present("debug") {
        println!("\nDump: {:?}", env);
    }
}

// Returns true if the index was manually updated
fn run_op(env: &mut Environment, ast: &Vec<Box<Op>>, ind: &mut usize) -> bool {
    let obj = ast[*ind].clone();
    match *obj {
        Op::Cmd(name, args) => {
            let shallow_ref: Vec<&Box<Op>> = args.iter().collect();
            run_cmd(env, ast, ind, &*name, &shallow_ref)
        },
        Op::Branch(_, body) => {
            for mut ind in 0..body.len() {
                if run_op(env, &body, &mut ind) {
                    return true;
                }
            }
            false
        }
        _ => panic!("Unimplemented top-level op: {:?}", ast[*ind]),
    }
}

// Converts op to a numeric value
fn to_numeric(env: &mut Environment, ast: &Vec<Box<Op>>, obj: &Box<Op>) -> i32 {
    match *obj.clone() {
        Op::Numeric(val) => val,
        // TODO: Implement Op::Memory for `to_numeric`
        Op::Memory(_) => panic!("Memory unimplemented"),
        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            if name.ends_with('x') {
                if name.len() == 3 {
                    env.registry.read_32(chrs[0], chrs[1])
                } else {
                    env.registry.read_16(chrs[0]) as i32
                }
            } else {
                if name.ends_with('h') {
                    env.registry.read_8(chrs[0], Position::Upper) as i32
                } else {
                    env.registry.read_8(chrs[0], Position::Lower) as i32
                }
            }
        }
        Op::Label(name) => {
            let moved: Vec<Op> = ast.iter().map(|x| *x.clone()).collect();
            moved
                .iter()
                .position(|entry| {
                    if let Op::Branch(n, _) = entry {
                        n[1..] == name[1..]
                    } else {
                        false
                    }
                })
                .unwrap() as i32
        }
        Op::BinOp(_, _, _) => panic!("Math unimplemented"),
        _ => panic!("Invalid numeric literal: {:?}", obj),
    }
}

// Pass in the op in which memory is modified, and it will automatically update it with the value
fn modify_memory(env: &mut Environment, ast: &Vec<Box<Op>>, obj: &Box<Op>, val: &Box<Op>) {
    match *obj.clone() {
        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            let val = to_numeric(env, &ast, val);
            if name.ends_with('x') {
                if chrs.len() == 3 {
                    env.registry.write_32(chrs[0], chrs[1], val);
                } else {
                    env.registry.write_16(chrs[0], val as i16);
                }
            } else {
                if name.ends_with('h') {
                    env.registry.write_8(chrs[0], Position::Upper, val as u8);
                } else {
                    env.registry.write_8(chrs[0], Position::Lower, val as u8);
                }
            }
        }
        Op::Memory(_) => {
            // TODO: Implement Op::Memory for `modify_memory`
            panic!("Memory unimplemented");
        }
        _ => panic!("Invalid parameter: {:?}", obj),
    }
}

// Returns `true` if `ind` was modified, `false` otherwise
fn run_cmd(
    env: &mut Environment,
    ast: &Vec<Box<Op>>,
    ind: &mut usize,
    cmd: &str,
    args: &Vec<&Box<Op>>,
) -> bool {
    // TODO: Add more commands
    match cmd {
        "mov" => {
            // Move second value into the first
            modify_memory(env, ast, args[0], args[1]);
            false
        }
        "inc" => {
            // new_val is 1 more than the previous value
            let new_val = Box::new(Op::Numeric(1 + to_numeric(env, ast, args[0])));
            modify_memory(env, ast, args[0], &new_val);
            false
        }
        "dec" => {
            // new_val is 1 less than the previous value
            let new_val = Box::new(Op::Numeric(to_numeric(env, ast, args[0]) - 1));
            modify_memory(env, ast, args[0], &new_val);
            false
        }
        "out" => {
            print!("{}", to_numeric(env, ast, args[0]));
            false
        }
        "goto" => {
            let i = to_numeric(env, ast, args[0]);
            *ind = i as usize;
            true
        }
        "mul" => {
            // args[0] * args[1] → args[0]
            let left = to_numeric(env, ast, args[0]);
            let right = to_numeric(env, ast, args[1]);

            modify_memory(
                env,
                ast,
                args[0],
                &Box::new(Op::Numeric(left * right)),
            );
            false
        }
        "div" => {
            // args[0] / args[1] → args[0]
            let left = to_numeric(env, ast, args[0]);
            let right = to_numeric(env, ast, args[1]);

            modify_memory(
                env,
                ast,
                args[0],
                &Box::new(Op::Numeric(left / right)),
            );
            false
        }
        "sub" => {
            // args[0] - args[1] → args[0]
            let left = to_numeric(env, ast, args[0]);
            let right = to_numeric(env, ast, args[1]);

            modify_memory(
                env,
                ast,
                args[0],
                &Box::new(Op::Numeric(left - right)),
            );
            false
        }
        "add" => {
            // args[0] + args[1] → args[0]
            let left = to_numeric(env, ast, args[0]);
            let right = to_numeric(env, ast, args[1]);

            modify_memory(
                env,
                ast,
                args[0],
                &Box::new(Op::Numeric(left + right)),
            );
            false
        }
        _ => panic!("Command: {} unrecognized", cmd),
    }
}
