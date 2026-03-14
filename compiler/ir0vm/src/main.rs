mod builtins;
mod cli;
mod error;
mod interpreter;
mod ir;
mod linker;
mod loader;
mod memory;
mod value;
mod vm;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let modules = loader::load_modules(&args.entry, &args.deps)?;
    let linked = linker::link_modules(modules)?;

    let mut vm = vm::Vm::new(linked);
    let mut interp = interpreter::Interpreter::new(&mut vm);

    let exit_code = interp.run_main()?;
    println!("program exited with: {}", exit_code);

    Ok(())
}
