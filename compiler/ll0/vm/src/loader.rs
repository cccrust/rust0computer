use crate::ir::ModuleRef;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn load_modules(entry: &Path, deps: &[PathBuf]) -> Result<Vec<ModuleRef>> {
    let mut modules = Vec::new();
    modules.push(load_module(entry)?);

    for dep in deps {
        modules.push(load_module(dep)?);
    }

    Ok(modules)
}

fn load_module(path: &Path) -> Result<ModuleRef> {
    let module = ModuleRef::from_ll_path(path)?;
    Ok(module)
}
