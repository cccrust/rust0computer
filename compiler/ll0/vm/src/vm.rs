#![allow(dead_code)]
use crate::linker::LinkedProgram;
use crate::memory::Memory;

#[derive(Debug)]
pub struct Vm {
    pub program: LinkedProgram,
    pub memory: Memory,
}

impl Vm {
    pub fn new(program: LinkedProgram) -> Self {
        Self {
            program,
            memory: Memory::default(),
        }
    }
}
