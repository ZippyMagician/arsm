pub mod consts;
pub mod iter;
pub mod mem;
pub mod token;
pub mod traits;

#[cfg(feature = "inline-python")]
use {
    crate::utils::{consts::REGISTER_REGEX, token::Op},
    pyo3::{prelude::*, types::IntoPyDict},
};

// I use Box::new a lot, so this makes it shorter
#[macro_export]
macro_rules! bx {
    ($value:expr) => {
        Box::new($value)
    };
}

#[cfg(feature = "inline-python")]
pub struct PyGuard {
    guard: GILGuard,
}

#[cfg(feature = "inline-python")]
impl PyGuard {
    pub fn new() -> Self {
        Self {
            guard: Python::acquire_gil(),
        }
    }

    #[inline]
    pub fn run_python(&self, env: &crate::env::Environment, code: &str) -> (Option<Vec<u8>>, i32) {
        let mut code = code.to_string();
        let py = self.guard.python();

        let old_stk = unsafe {
            env.mem.read_range(
                crate::utils::consts::OFFSET..env.mem.s_len + crate::utils::consts::OFFSET,
            )
        };

        if REGISTER_REGEX.is_match(&code) {
            // We can pass an empty slice for the AST as we know this will always be simply a register
            code = REGISTER_REGEX
                .replace_all(&code, |caps: &regex::Captures<'_>| {
                    crate::parser::to_numeric::<i32>(
                        &mut env.shallow_copy(),
                        &[],
                        &Op::Register(format!("{}", &caps[1])),
                    )
                    .to_string()
                })
                .to_string();
        }

        let dict = [("stk", old_stk)].into_py_dict(py);

        let prog = format!(
            "{}(\"{}\")",
            self::consts::PYTHON_HEAD,
            code.trim().escape_default()
        );

        py.run(&*prog, None, Some(dict)).unwrap_or_else(|e| {
            e.print_and_set_sys_last_vars(py);
            std::process::exit(1);
        });

        // Return new stack and return value of inline code
        let new_stk = dict.get_item("stk").unwrap().extract::<Vec<u8>>().unwrap();
        (
            if old_stk.iter().copied().collect::<Vec<u8>>() == new_stk {
                None
            } else {
                Some(new_stk)
            },
            dict.get_item("ret")
                .expect("Could not run python code")
                .extract()
                .unwrap_or_default(),
        )
    }
}
