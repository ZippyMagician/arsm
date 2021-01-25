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

// Proof of concept. Once I fix linking, update this to return the value, and take in the memory's registers
#[cfg(feature = "inline-python")]
pub fn run_python(code: &str) {
    Python::with_gil(|py| {
        (|| -> PyResult<()> {
            let res: String = py.eval(code, None, None)?.extract::<String>()?;
            println!("Result: {}", res);
            Ok(())
        })()
        .map_err(|e| {
            e.print_and_set_sys_last_vars(py);
        })
    })
    .expect("Could not create python instance");
}
