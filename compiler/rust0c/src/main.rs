mod ast;
mod cli;
mod codegen;
mod error;
mod ir;
mod lexer;
mod parser;
mod sema;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let source = std::fs::read_to_string(&args.input)?;
    let tokens = lexer::lex(&source)?;
    let ast = parser::parse(&tokens)?;
    let checked = sema::analyze(ast)?;
    let ir = codegen::lower_to_ir(checked)?;

    std::fs::write(&args.output, ir.to_string())?;
    println!("wrote {}", args.output.display());

    Ok(())
}
