use crate::mem::Memory;
use crate::utils::{iter::BufIter, token::Op};

// Simple environment structure that holds the memory and registry
// TODO: Stack
#[derive(Debug, PartialEq)]
pub struct Environment {
    pub mem: Memory,

    pub stdin: BufIter<u8>,

    parent_ast: Option<Vec<Op>>,
}

impl Environment {
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            mem: Memory::init(1024, 20),
            stdin: BufIter::new(buf),
            parent_ast: None,
        }
    }

    pub fn set_parent(&mut self, ast: &[Op]) {
        self.parent_ast = Some(ast.to_owned());
    }

    pub fn get_parent(&self) -> &Option<Vec<Op>> {
        &self.parent_ast
    }

    pub fn clear_parent(&mut self) {
        self.parent_ast = None;
    }
}
