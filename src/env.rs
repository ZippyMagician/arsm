use crate::utils::mem::Memory;
use crate::utils::{iter::BufIter, token::Op};

// Simple environment structure that holds the memory, stdin, and a few useful items
// TODO: Stack
#[derive(Debug, PartialEq)]
pub struct Environment {
    pub mem: Memory,

    pub stdin: BufIter<u8>,

    parent_ast: Option<Vec<Op>>,

    pub jump_point: Vec<(usize, usize)>,

    pub pos: (usize, usize),
}

impl Environment {
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            mem: Memory::init(1024, 0),
            stdin: BufIter::new(buf),
            parent_ast: None,
            jump_point: Vec::new(),
            pos: (0, 0),
        }
    }

    pub fn set_parent(&mut self, ast: &[Op]) {
        self.parent_ast = Some(ast.to_owned());
    }

    pub fn get_parent(&self) -> &Option<Vec<Op>> {
        &self.parent_ast
    }
}
