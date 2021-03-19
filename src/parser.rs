use clap::ArgMatches;

use crate::utils::Num;
use crate::utils::{token::Op, traits::*};
use crate::{bx, env::Environment};

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

// Incorrect casts will not error, instead running with undefined behaviour
// S is technically only necessary if reading from the stack, however it is unfortunately impossible to tell the compiler that
pub fn to_numeric<S: Size>(env: &mut Environment, ast: &[Op], obj: &Op) -> Num {
    match obj {
        Op::Numeric(val) => Num { i32: *val },

        Op::StackMarker => {
            match S::len() {
                1 => Num {
                    u8: env
                        .mem
                        .s_pop_8()
                        .expect("Attempted to pop from empty stack"),
                },
                2 => Num {
                    i16: env
                        .mem
                        .s_pop_16()
                        .expect("Attempted to pop from empty stack"),
                },
                4 | 8 => Num {
                    i32: env
                        .mem
                        .s_pop_32()
                        .expect("Attempted to pop from empty stack"),
                },
                // Can't happen
                _ => panic!(),
            }
        }

        Op::Memory(ident, op) => {
            let val = unsafe { to_numeric::<usize>(env, ast, &op).usize };

            match ident {
                '#' => env.mem.m_read::<u8>(val),
                '$' => env.mem.m_read::<i16>(val),
                '@' => env.mem.m_read::<i32>(val),
                _ => panic!("Invalid memory identifier: '{}'", ident),
            }
        }

        Op::Register(name) => {
            let chrs: Vec<char> = name.chars().collect();
            if name.ends_with('x') {
                if name.len() == 3 {
                    env.mem.r_read(&(chrs[0], chrs[1]))
                } else {
                    env.mem.r_read(&chrs[0])
                }
            } else if name.ends_with('h') {
                env.mem.r_read(&(chrs[0], Pos::Upper))
            } else {
                env.mem.r_read(&(chrs[0], Pos::Lower))
            }
        }

        Op::Label(name) => Num {
            usize: env
                .get_parent()
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
        },

        Op::BinOp(..) => todo!("Math unimplemented"),

        Op::Char(chr) => Num { u8: *chr as u8 },

        Op::Cmd(name, args) => {
            let mut dummy_ind = 0;
            let args: Vec<&Op> = args.iter().collect();
            Num {
                i32: run_cmd(env, ast, &mut dummy_ind, &*name, &args).get_val(),
            }
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

            Num { i32: res }
        }

        _ => panic!("Invalid numeric literal: {:?}", obj),
    }
}

// Pass in the op in which memory is modified, and it will automatically update
// it with the value
fn modify_memory(env: &mut Environment, ast: &[Op], obj: &Op, val: &Op) {
    match obj {
        Op::Register(name) => {
            let chrs = name.chars().collect::<Vec<char>>();
            if name.ends_with('x') {
                if chrs.len() == 3 {
                    let val = to_numeric::<i32>(env, &ast, val);
                    env.mem.r_write(&(chrs[0], chrs[1]), &val);
                } else {
                    let val = to_numeric::<i16>(env, &ast, val);
                    env.mem.r_write(&chrs[0], &val);
                }
            } else if name.ends_with('h') {
                let val = to_numeric::<u8>(env, &ast, val);
                env.mem.r_write(&(chrs[0], Pos::Upper), &val);
            } else {
                let val = to_numeric::<u8>(env, &ast, val);
                env.mem.r_write(&(chrs[0], Pos::Lower), &val);
            }
        }

        Op::Memory(ident, op) => {
            let pos = unsafe { to_numeric::<usize>(env, &ast, &op).usize };
            match ident {
                '#' => {
                    let val = to_numeric::<u8>(env, &ast, val);
                    env.mem.m_write::<u8>(pos, &val);
                }

                '$' => {
                    let val = to_numeric::<i16>(env, &ast, val);
                    env.mem.m_write::<i16>(pos, &val);
                }

                '@' => {
                    let val = to_numeric::<i32>(env, &ast, val);
                    env.mem.m_write::<i32>(pos, &val);
                }

                _ => panic!("Invalid identifier for memory: '{}'", ident),
            }
        }

        _ => panic!("Invalid parameter: {:?}", obj),
    }
}

#[inline(always)]
fn set_ind(ind: &mut usize, env: &mut Environment, val: usize) {
    *ind = val;
    env.jump_point.push(env.pos);
    env.pos = (*ind, 0);
}

#[inline]
pub fn check_cmp<T>(env: &mut Environment, ast: &[Op], args: &[&Op], f: T) -> Box<dyn Status>
where
    T: FnOnce(i32, i32) -> bool,
{
    let left = to_numeric::<i32>(env, ast, args[0]);
    let right = to_numeric::<i32>(env, ast, args[1]);

    if unsafe { f(left.i32, right.i32) } {
        env.mem.flag_write_cmp();
    } else {
        env.mem.flag_reset_cmp();
    }

    bx!(false)
}

#[inline]
pub fn perform_op<T>(
    env: &mut Environment,
    ast: &[Op],
    args: &[&Op],
    cmd: &str,
    f: T,
) -> Box<dyn Status>
where
    T: FnOnce(i32, i32) -> i32,
{
    if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
        return bx!(false);
    }

    let left = to_numeric::<i32>(env, ast, args[0]);
    let right = to_numeric::<i32>(env, ast, args[1]);
    modify_memory(
        env,
        ast,
        args[0],
        &Op::Numeric(unsafe { f(left.i32, right.i32) }),
    );
    bx!(false)
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
        "mov" | "cmo" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            // Move second value into the first
            modify_memory(env, ast, args[0], args[1]);
            bx!(false)
        }

        "inc" | "cin" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            // new_val is 1 more than the previous value
            let new_val = Op::Numeric(1 + unsafe { to_numeric::<i32>(env, ast, args[0]).i32 });
            modify_memory(env, ast, args[0], &new_val);
            bx!(false)
        }

        "dec" | "cde" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            // new_val is 1 less than the previous value
            let new_val = Op::Numeric(unsafe { to_numeric::<i32>(env, ast, args[0]).i32 } - 1);
            modify_memory(env, ast, args[0], &new_val);
            bx!(false)
        }

        "out" | "cou" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            print!("{}", to_numeric::<i32>(env, ast, args[0]));
            bx!(false)
        }

        "chr" | "cch" => {
            if cmd == "cch" && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            print!("{}", unsafe { to_numeric::<u8>(env, ast, args[0]).u8 }
                as char);
            bx!(false)
        }

        "jmp" | "cjm" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            let n = unsafe { to_numeric::<usize>(env, ast, args[0]).usize };
            set_ind(ind, env, n);
            bx!(true)
        }

        "mul" | "cmu" => perform_op(env, ast, args, cmd, |l, r| l * r),

        "div" | "cdi" => perform_op(env, ast, args, cmd, |l, r| l / r),

        "sub" | "csu" => perform_op(env, ast, args, cmd, |l, r| l - r),

        "add" | "cad" => perform_op(env, ast, args, cmd, |l, r| l + r),

        "ceq" => check_cmp(env, ast, args, |l, r| l == r),

        "cne" => check_cmp(env, ast, args, |l, r| l != r),

        "cl" => check_cmp(env, ast, args, |l, r| l < r),

        "cle" => check_cmp(env, ast, args, |l, r| l <= r),

        "cg" => check_cmp(env, ast, args, |l, r| l > r),

        "cge" => check_cmp(env, ast, args, |l, r| l >= r),

        "cz" => {
            if unsafe { to_numeric::<i32>(env, ast, args[0]).i32 } == 0 {
                env.mem.flag_write_cmp();
            } else {
                env.mem.flag_reset_cmp();
            }

            bx!(false)
        }

        "str" => match args[0] {
            Op::String(val) => {
                for (i, chr) in val.chars().enumerate() {
                    env.mem.m_write::<u8>(i, &Num { u8: chr as u8 });
                }

                let terminator = to_numeric::<u8>(env, ast, args[1]);
                env.mem.m_write::<u8>(val.len(), &terminator);
                bx!(false)
            }

            _ => panic!(
                "Argument #0 for command 'str' must be of type Op::String. Instead got: {:?}",
                ast[*ind]
            ),
        },

        "stk" => {
            let count = unsafe { to_numeric::<usize>(env, ast, args[0]).usize };
            env.mem.resize_stack(count);
            bx!(false)
        }

        "psh" | "cps" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            let allocation = unsafe { to_numeric::<usize>(env, ast, args[0]).usize };
            match allocation {
                1 => {
                    let n = to_numeric::<u8>(env, ast, args[1]);
                    env.mem.s_push::<u8>(&n)
                }

                2 => {
                    let n = to_numeric::<i16>(env, ast, args[1]);
                    env.mem.s_push::<i16>(&n)
                }

                4 => {
                    let n = to_numeric::<i32>(env, ast, args[1]);
                    env.mem.s_push::<i32>(&n)
                }
                // Shouldn't happen
                _ => panic!(),
            }

            bx!(false)
        }

        "pop" | "cpo" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            modify_memory(env, ast, args[0], &Op::StackMarker);
            bx!(false)
        }

        "lsh" | "cls" => perform_op(env, ast, args, cmd, |l, r| l << r),

        "rsh" | "crs" => perform_op(env, ast, args, cmd, |l, r| l >> r),

        "or" | "cor" => perform_op(env, ast, args, cmd, |l, r| l | r),

        "xor" | "cxo" => perform_op(env, ast, args, cmd, |l, r| l ^ r),

        "and" | "can" => perform_op(env, ast, args, cmd, |l, r| l & r),

        "not" | "cno" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            let val = unsafe { to_numeric::<i32>(env, ast, args[0]).i32 };
            modify_memory(env, ast, args[0], &Op::Numeric(!val));
            bx!(false)
        }

        "swp" | "csw" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            let val = args[1];
            modify_memory(env, ast, args[1], args[0]);
            modify_memory(env, ast, args[0], val);
            bx!(false)
        }

        "db" => {
            let mut i = unsafe { to_numeric::<usize>(env, ast, args[0]).usize };
            let terminator = unsafe { to_numeric::<u8>(env, ast, args[1]).u8 };
            let mut len = 0;
            while unsafe { env.mem.m_read::<u8>(i).u8 } != terminator {
                len += 1;
                i += 1;
            }
            bx!(len)
        }

        "in" => bx!(match env.stdin.next() {
            Some(val) => i32::from(val),
            None => 0,
        }),

        "ret" | "cre" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            if !env.jump_point.is_empty() {
                let (mut left, right) = env.jump_point.pop().unwrap();
                // If it a top-level call, return to the next bit of the top-level. Otherwise, return to the next bit of the branch
                if let Op::Branch(..) = env.get_parent().as_ref().unwrap()[left] {
                    env.pos = (left, right + 1);
                } else {
                    left += 1;
                    env.pos = (left, right);
                }
                *ind = left;
            } else {
                panic!("Cannot return");
            }

            bx!(true)
        }

        "hlt" | "chl" => {
            if cmd.starts_with('c') && !env.mem.flag_read_cmp() {
                return bx!(false);
            }

            std::process::exit(unsafe { to_numeric::<i32>(env, ast, args[0]).i32 })
        }

        _ => panic!("Command: {} unrecognized", cmd),
    }
}
