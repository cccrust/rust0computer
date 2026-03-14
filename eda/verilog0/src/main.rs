mod lexer;
mod parser;
mod elaborator;
mod codegen;
mod vm; // 1. 加入 vm 模組

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

/// rverilog - A Verilog compiler generating VVP-compatible bytecode
#[derive(ClapParser, Debug)]
#[command(
    name = "rverilog",
    about = "Verilog compiler that generates VVP bytecode (like iverilog)",
    version = "0.1.0"
)]
struct Args {
    /// Input Verilog source file(s)
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output file (default: a.vvp)
    #[arg(short = 'o', long = "output", default_value = "a.vvp")]
    output: PathBuf,

    /// Top-level module name
    #[arg(short = 's', long = "top")]
    top_module: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Only parse and check syntax, don't generate output
    #[arg(long = "check")]
    check_only: bool,

    /// Dump AST (for debugging)
    #[arg(long = "dump-ast")]
    dump_ast: bool,

    /// 2. 新增 --run 參數
    #[arg(short = 'r', long = "run")]
    run_vm: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        eprintln!("rverilog v0.1.0 - Rust Verilog Compiler");
    }

    // Read and concatenate all input files
    let mut combined_source = String::new();
    for path in &args.input {
        match fs::read_to_string(path) {
            Ok(src) => {
                if args.verbose {
                    eprintln!("  Reading: {}", path.display());
                }
                combined_source.push_str(&src);
                combined_source.push('\n');
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    // === PHASE 1: Lexing ===
    if args.verbose {
        eprintln!("[1/4] Lexing...");
    }
    let tokens = match lexer::tokenize(&combined_source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    // === PHASE 2: Parsing ===
    if args.verbose {
        eprintln!("[2/4] Parsing...");
    }
    let ast = match parser::parse(tokens) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    if args.dump_ast {
        println!("{:#?}", ast);
    }

    if args.check_only {
        println!("Syntax OK");
        return;
    }

    // === PHASE 3: Elaboration ===
    if args.verbose {
        eprintln!("[3/4] Elaborating...");
    }
    let design = match elaborator::elaborate(ast, args.top_module.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Elaboration error: {}", e);
            std::process::exit(1);
        }
    };

    // === PHASE 4: Code Generation ===
    if args.verbose {
        eprintln!("[4/4] Generating VVP bytecode...");
    }
    let vvp_code = match codegen::generate(&design) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Write output
    match fs::write(&args.output, &vvp_code) {
        Ok(_) => {
            if args.verbose {
                eprintln!("Output written to: {}", args.output.display());
            }
        }
        Err(e) => {
            eprintln!("Error writing output: {}", e);
            std::process::exit(1);
        }
    }

    println!(
        "Compiled successfully -> {} ({} bytes)",
        args.output.display(),
        vvp_code.len()
    );

    // === PHASE 5: VM Execution ===
    // 3. 如果有帶 --run，則啟動 VM 執行剛剛生成的代碼
    if args.run_vm {
        if args.verbose {
            eprintln!("[5/5] Running VM...");
        }
        let mut virtual_machine = vm::Vm::new();
        if let Err(e) = virtual_machine.load_from_string(&vvp_code) {
            eprintln!("VM Load error: {}", e);
            std::process::exit(1);
        }
        virtual_machine.run();
    }
}
