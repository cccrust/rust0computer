use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "LLVM IR (.ll) virtual machine", author, version)]
pub struct Args {
    /// Entry .ll file (contains main)
    pub entry: PathBuf,

    /// Dependent .ll files to link
    pub deps: Vec<PathBuf>,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
