use std::fmt::{self, Debug, Formatter};

#[cfg(feature = "inline-python")]
use crate::utils::PyGuard;
use crate::utils::{iter::BufIter, mem::Memory, token::Op};

// Simple environment structure that holds the memory, stdin, and a few useful items
pub struct Environment {
    pub mem: Memory,

    pub stdin: BufIter<u8>,

    parent_ast: Option<Vec<Op>>,

    pub jump_point: Vec<(usize, usize)>,

    pub pos: (usize, usize),

    #[cfg(feature = "inline-python")]
    pub py: PyGuard,
}

impl Environment {
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            mem: Memory::init(1024, 0),
            stdin: BufIter::new(buf),
            parent_ast: None,
            jump_point: Vec::with_capacity(5),
            pos: (0, 0),
            #[cfg(feature = "inline-python")]
            py: PyGuard::new(),
        }
    }

    pub fn set_parent(&mut self, ast: &[Op]) {
        self.parent_ast = Some(ast.to_owned());
    }

    pub fn get_parent(&self) -> &Option<Vec<Op>> {
        &self.parent_ast
    }

    // Only used by inline-python
    #[cfg(feature = "inline-python")]
    pub fn shallow_copy(&self) -> Self {
        Self {
            mem: self.mem,
            stdin: BufIter::new(&mut []),
            parent_ast: None,
            jump_point: Vec::with_capacity(5),
            pos: (0, 0),
            #[cfg(feature = "inline-python")]
            py: PyGuard::new(),
        }
    }
}

impl Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("mem", &self.mem)
            .field("stdin", &self.stdin)
            .finish()
    }
}
