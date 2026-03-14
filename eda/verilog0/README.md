# rverilog — Rust Verilog Compiler

A Verilog compiler written in Rust that produces **VVP bytecode**, compatible with the `vvp` runtime from [Icarus Verilog](http://iverilog.icarus.com/). The goal is feature parity with `iverilog` for synthesizable + behavioral Verilog (IEEE 1364-2001).

```
  .v source files
       │
       ▼
  ┌─────────┐
  │  Lexer  │  tokenize() → Vec<Token>
  └────┬────┘
       │
       ▼
  ┌─────────┐
  │  Parser │  parse()    → AST (SourceFile)
  └────┬────┘
       │
       ▼
  ┌─────────────┐
  │ Elaborator  │  elaborate() → Design (typed, resolved)
  └──────┬──────┘
         │
         ▼
  ┌──────────────┐
  │ Code Generator│ generate() → VVP text
  └──────────────┘
         │
         ▼
     a.vvp file  ──►  vvp a.vvp  (simulation)
```

## Building

```bash
# Requires Rust 1.70+ and cargo
cargo build --release

# Binary will be at:
# ./target/release/rverilog
```

## Usage

```bash
# Basic compilation
rverilog design.v -o simulation.vvp

# Multiple source files
rverilog top.v submodule.v pkg.v -o sim.vvp

# Specify top-level module
rverilog top.v -s tb_top -o sim.vvp

# Check syntax only (no output generated)
rverilog design.v --check

# Dump the AST for debugging
rverilog design.v --dump-ast

# Verbose output
rverilog design.v -v -o sim.vvp

# Then run with vvp (from iverilog package)
vvp sim.vvp
```

## Supported Verilog Features

### Declarations
- `module` / `endmodule`
- Port directions: `input`, `output`, `inout`
- Net types: `wire`, `reg`, `integer`, `real`
- `parameter`, `localparam`
- Vector ranges: `[7:0]`, `[MSB:LSB]`
- `signed`/`unsigned` qualifiers

### Behavioral Constructs
- `always @(*)`, `always @(posedge clk)`, `always @(negedge rst)`
- `initial` blocks
- `begin`/`end` sequential blocks
- `fork`/`join` parallel blocks
- `if`/`else`
- `case`, `casex`, `casez`, `endcase`
- `for`, `while`, `repeat`, `forever` loops
- Blocking assignments: `=`
- Non-blocking assignments: `<=`
- `#delay` timing control
- `@event` sensitivity control
- `disable`

### Structural Constructs
- Continuous `assign`
- Module instantiation with positional and named port connections
- Parameter overrides `#(.PARAM(val))`
- Primitive gates: `and`, `or`, `not`, `nand`, `nor`, `xor`, `xnor`, `buf`

### Expressions
- All arithmetic: `+`, `-`, `*`, `/`, `%`
- All comparisons: `==`, `!=`, `===`, `!==`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- Bitwise: `&`, `|`, `^`, `~^`, `~`
- Reduction: `&expr`, `|expr`, `^expr`
- Shift: `<<`, `>>`, `<<<`, `>>>`
- Ternary: `cond ? a : b`
- Concatenation: `{a, b, c}`
- Replication: `{4{a}}`
- Based literals: `8'b1010`, `4'hF`, `16'd255`

### System Tasks
- `$display`, `$write`, `$strobe`, `$monitor`
- `$finish`, `$stop`
- `$time`, `$realtime`
- `$dumpvars`, `$dumpfile`, `$dumpall`, `$dumpflush`
- `$random`
- `$readmemb`, `$readmemh`

## VVP Output Format

The compiler generates VVP bytecode text format compatible with Icarus Verilog's `vvp` runtime. The format includes:

- **Net declarations** (`.net`, `.var`) for wire and reg signals
- **Functor chains** (`.and`, `.or`, `.xor`, `.arith/sum`, etc.) for continuous logic
- **Thread blocks** for `always` and `initial` procedural code
- **VPI calls** for system tasks (`%vpi_call`)
- **VVP instructions** like `%load/vec4`, `%assign/wr`, `%assign/nb`, `%jmp`, etc.

## Project Structure

```
src/
├── main.rs          # CLI entry point (clap-based)
├── lexer/
│   └── mod.rs       # Hand-written lexer — tokenizes Verilog source
├── parser/
│   └── mod.rs       # Recursive descent parser — produces AST
├── elaborator/
│   └── mod.rs       # Semantic analysis, symbol resolution
└── codegen/
    └── mod.rs       # VVP bytecode emitter
```

## Example

```verilog
// counter.v
module counter(
    input clk,
    input rst,
    output reg [7:0] count
);
    always @(posedge clk or posedge rst) begin
        if (rst)
            count <= 8'b0;
        else
            count <= count + 1;
    end
endmodule
```

```bash
rverilog counter.v -o counter.vvp
vvp counter.vvp
```

## Limitations / TODO

- [ ] Full VVP binary format (currently emits text/assembly form)  
- [ ] `generate` blocks with loop unrolling  
- [ ] `task` and `function` full elaboration  
- [ ] `defparam` overrides  
- [ ] Tri-state / `z` value tracking  
- [ ] Timing checks (`$setup`, `$hold`, etc.)  
- [ ] SystemVerilog extensions  
- [ ] `include` / `define` preprocessor (basic support only)  
- [ ] Multi-driver resolution  

## Contributing

Pull requests welcome! Especially for:
- More complete VVP instruction set coverage
- Better error messages with source locations
- SystemVerilog features
- Test suite with known-good iverilog outputs

## License

MIT
