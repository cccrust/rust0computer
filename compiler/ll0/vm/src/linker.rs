use crate::ir::ModuleRef;
use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct LinkedProgram {
    pub modules: Vec<ModuleRef>,
    pub functions: HashMap<String, FunctionRef>,
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionRef {
    pub module_index: usize,
    pub function_index: usize,
}

pub fn link_modules(modules: Vec<ModuleRef>) -> Result<LinkedProgram> {
    let mut functions = HashMap::new();

    for (m_idx, m) in modules.iter().enumerate() {
        for (f_idx, f) in m.module.functions.iter().enumerate() {
            if functions.contains_key(&f.name) {
                bail!("duplicate function definition: {}", f.name);
            }
            functions.insert(
                f.name.clone(),
                FunctionRef {
                    module_index: m_idx,
                    function_index: f_idx,
                },
            );
        }
    }

    Ok(LinkedProgram { modules, functions })
}
