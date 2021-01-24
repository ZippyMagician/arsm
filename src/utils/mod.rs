pub mod consts;
pub mod iter;
pub mod mem;
pub mod token;
pub mod traits;

// I use Box::new a lot, so this makes it shorter
#[macro_export]
macro_rules! bx {
	($value:expr) => {
		Box::new($value)
	};
}
