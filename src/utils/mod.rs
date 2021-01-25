pub mod consts;
pub mod iter;
pub mod mem;
pub mod token;
pub mod traits;

#[cfg(feature = "inline-python")]
use pyo3::prelude::*;

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
    pub fn run_python(&self, code: &str) -> i32 {
        let py = self.guard.python();
        py.eval(code, None, None)
            .unwrap_or_else(|e| {
                e.print_and_set_sys_last_vars(py);
                std::process::exit(1)
            })
            .extract()
            .unwrap_or_default()
    }
}
