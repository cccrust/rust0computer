use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "Toy Rust subset compiler", author, version)]
pub struct Args {
    /// Input Rust source file
    pub input: PathBuf,

    /// Output LLVM IR (.ll) file
    #[arg(short, long, default_value = "out.ll")]
    pub output: PathBuf,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
