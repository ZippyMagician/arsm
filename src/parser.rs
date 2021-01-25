use clap::ArgMatches;

use crate::bx;
use crate::env::Environment;
use crate::utils::{token::Op, traits::*};

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

    let mut env = Environment::new(&mut v);
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
            // If we are not in a branch, clear the jump_points for performance
            if !env.jump_point.is_empty() && ast == env.get_parent().clone().unwrap() {
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
                }

                env.pos.1 += 1;
            }

            bx!(false)
        }

        #[cfg(feature = "inline-python")]
        Op::InlinePy(code) => {
            let (new_stk, _) = env.py.run_python(env, code);
            // Update the stack
            if let Some(new_stk) = new_stk {
                env.mem.write_range(
                    crate::utils::consts::OFFSET..env.mem.s_size + crate::utils::consts::OFFSET,
                    &new_stk,
                );
                env.mem.s_len = new_stk.len();
                env.mem.s_size = env.mem.s_len.max(env.mem.s_size);
            }

            bx!(false)
        }

        _ => panic!("Invalid top-level op: {:?}", ast[*ind]),
    }
}

// Converts op to a numeric value
fn to_numeric<T: Num + Clone>(env: &mut Environment, ast: &[Op], obj: &Op) -> T {
    match obj {
        Op::Numeric(val) => num_traits::cast(*val),

        Op::StackMarker => {
            match T::len() {
                1 => num_traits::cast(
                    env.mem
                        .s_pop_8()
                        .expect("Attempted to pop from empty stack"),
                ),
                2 => num_traits::cast(
                    env.mem
                        .s_pop_16()
                        .expect("Attempted to pop from empty stack"),
                ),
                4 => num_traits::cast(
                    env.mem
                        .s_pop_32()
                        .expect("Attempted to pop from empty stack"),
                ),
                8 => num_traits::cast(
                    env.mem
                        .s_pop_32()
                        .expect("Attempted to pop from empty stack"),
                ),
                // Can't happen
                _ => panic!(),
            }
        }

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
            let chrs: Vec<char> = name.chars().collect();
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
                .unwrap_or_else(|| panic!("No matching branch for label {}", name)),
        ),

        Op::BinOp(..) => todo!("Math unimplemented"),

        Op::Char(chr) => num_traits::cast(*chr as u8),

        Op::Cmd(name, args) => {
            let mut dummy_ind = 0;
            let args: Vec<&Op> = args.iter().collect();
            num_traits::cast(run_cmd(env, ast, &mut dummy_ind, &*name, &args).get_val())
        }

        #[cfg(feature = "inline-python")]
        Op::InlinePy(code) => {
            let (new_stk, res) = env.py.run_python(env, code);
            // Update the stack
            if let Some(new_stk) = new_stk {
                env.mem.write_range(
                    crate::utils::consts::OFFSET..env.mem.s_size + crate::utils::consts::OFFSET,
                    &new_stk,
                );
                env.mem.s_len = new_stk.len();
                env.mem.s_size = env.mem.s_len.max(env.mem.s_size);
            }
            num_traits::cast(res)
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

// Pass in the op in which memory is modified, and it will automatically update
// it with the value
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

#[inline]
fn set_ind(ind: &mut usize, env: &mut Environment, val: usize) {
    *ind = val;
    env.jump_point.push(env.pos);
    env.pos = (*ind, 0);
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
            let n = to_numeric(env, ast, args[0]);
            set_ind(ind, env, n);
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

        "cmp" => {
            env.mem.flag_reset_cmp();

            let left: i32 = to_numeric(env, ast, args[0]);
            let right: i32 = to_numeric(env, ast, args[1]);

            let mut num = 0;
            if left == right {
                num |= 0b10;
                if right == 0 {
                    num |= 1
                }
            }
            if left > right {
                num |= 0b100
            }
            if left < right {
                num |= 0b1000
            }
            env.mem.flag_write_whole_cmp(num);

            bx!(false)
        }

        "je" => {
            bx!(if env.mem.flag_read_cmp(1) {
                let n = to_numeric(env, ast, args[0]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jne" => {
            bx!(if !env.mem.flag_read_cmp(1) {
                let n = to_numeric(env, ast, args[2]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jz" => {
            bx!(if to_numeric::<i32>(env, ast, args[0]) == 0 {
                let n = to_numeric(env, ast, args[1]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jg" => {
            bx!(if env.mem.flag_read_cmp(2) {
                let n = to_numeric(env, ast, args[0]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jge" => {
            bx!(if env.mem.flag_read_cmp(1) && env.mem.flag_read_cmp(2) {
                let n = to_numeric(env, ast, args[0]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jl" => {
            bx!(if env.mem.flag_read_cmp(3) {
                let n = to_numeric(env, ast, args[0]);
                set_ind(ind, env, n);
                true
            } else {
                false
            })
        }

        "jle" => {
            bx!(if env.mem.flag_read_cmp(1) && env.mem.flag_read_cmp(3) {
                let n = to_numeric(env, ast, args[0]);
                set_ind(ind, env, n);
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

        "stk" => {
            let count = to_numeric(env, ast, args[0]);
            env.mem.resize_stack(count);
            bx!(false)
        }

        "psh" => {
            let allocation = to_numeric(env, ast, args[0]);
            match allocation {
                1 => {
                    let n = to_numeric(env, ast, args[1]);
                    env.mem.s_push::<u8>(&n)
                }

                2 => {
                    let n = to_numeric(env, ast, args[1]);
                    env.mem.s_push::<i16>(&n)
                }

                4 => {
                    let n = to_numeric(env, ast, args[1]);
                    env.mem.s_push::<i32>(&n)
                }
                // Shouldn't happen
                _ => panic!(),
            }

            bx!(false)
        }

        "pop" => {
            modify_memory(env, ast, args[0], &Op::StackMarker);
            bx!(false)
        }

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
            if !env.jump_point.is_empty() {
                let (left, right) = env.jump_point.pop().unwrap();
                // If it a top-level call, return to the next bit of the top-level. Otherwise, return to the next bit of the branch
                if let Op::Branch(_, _) = env.get_parent().as_ref().unwrap()[left] {
                    env.pos = (left, right + 1);
                } else {
                    env.pos = (left + 1, right);
                }
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
