use crate::registry::Register;

// Simple environment structure that holds the memory and registry
// TODO: Stack
#[derive(Debug, PartialEq)]
pub struct Environment {
    pub mem: [u8; 1024],

    pub registry: Register,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            mem: [0u8; 1024],
            registry: Register::init(),
        }
    }
}
