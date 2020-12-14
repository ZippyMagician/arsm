use crate::iter::BufIter;
use crate::registry::Register;
use crate::token::Op;

// Simple environment structure that holds the memory and registry
// TODO: Stack
#[derive(Debug, PartialEq)]
pub struct Environment {
    pub mem: [u8; 1024],

    pub registry: Register,

    pub stdin: BufIter<u8>,

    parent_ast: Option<Vec<Op>>,
}

impl Environment {
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            mem: [0u8; 1024],
            registry: Register::init(),
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
