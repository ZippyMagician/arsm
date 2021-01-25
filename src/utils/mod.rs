pub mod consts;
pub mod iter;
pub mod mem;
pub mod token;
pub mod traits;

#[cfg(feature = "inline-python")]
use pyo3::{prelude::*, types::IntoPyDict};

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
    pub fn run_python(&self, env: &crate::env::Environment, code: &str) -> i32 {
        let py = self.guard.python();
        let dict = [("stk", unsafe {
            env.mem.read_range(crate::utils::consts::OFFSET..env.mem.s_len + crate::utils::consts::OFFSET)
        })];

        py.eval(code.trim(), None, Some(dict.into_py_dict(py)))
            .unwrap_or_else(|e| {
                e.print_and_set_sys_last_vars(py);
                std::process::exit(1)
            })
            .extract()
            .unwrap_or_default()
    }
}
