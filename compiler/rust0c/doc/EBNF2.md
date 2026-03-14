# Rust EBNF (Compatible Subset)

This document defines a **strict Rust subset** intended to be **fully accepted
by `rustc`** (Rust 2021/2024) when programs follow the listed grammar **and**
semantic constraints.

The goal is compatibility and implementability, not full language coverage.

## Lexical

```ebnf
Ident       = Letter, { Letter | Digit | "_" } ;
Int         = Digit, { Digit } ;

Letter      = "A".."Z" | "a".."z" | "_" ;
Digit       = "0".."9" ;
```

## Program

```ebnf
Program     = { Function } ;

Function    = "fn", Ident, "(", [ ParamList ], ")", "->", Type, Block ;
ParamList   = Param, { ",", Param } ;
Param       = Ident, ":", Type ;

Type        = "i32" | "bool" ;
```

## Statements and Blocks

```ebnf
Block       = "{", { Stmt }, Expr, "}" ;
Stmt        = LetStmt | ExprStmt ;

LetStmt     = "let", Ident, ":", Type, "=", Expr, ";" ;
ExprStmt    = Expr, ";" ;
```

## Expressions (precedence)

```ebnf
Expr        = IfExpr | WhileExpr | OrExpr ;

IfExpr      = "if", Expr, Block, "else", Block ;
WhileExpr   = "while", Expr, Block ;

OrExpr      = AndExpr, { "||", AndExpr } ;
AndExpr     = EqExpr,  { "&&", EqExpr } ;
EqExpr      = CmpExpr, { ( "==" | "!=" ), CmpExpr } ;
CmpExpr     = AddExpr, { ( "<" | "<=" | ">" | ">=" ), AddExpr } ;
AddExpr     = MulExpr, { ( "+" | "-" ), MulExpr } ;
MulExpr     = UnaryExpr, { ( "*" | "/" | "%" ), UnaryExpr } ;

UnaryExpr   = ( "!" | "-" ), UnaryExpr
            | PostfixExpr ;

PostfixExpr = PrimaryExpr, { Call } ;
Call        = "(", [ ExprList ], ")" ;

PrimaryExpr = Int
            | "true"
            | "false"
            | Ident
            | "(", Expr, ")"
            | Block ;

ExprList    = Expr, { ",", Expr } ;
```

## Semantic Constraints (required for rustc compatibility)

1. **No reassignment**: `let` declares a new immutable binding. No `mut`.
2. **No shadowing inside the same block**: each `Ident` in `let` is unique per block.
3. **Only `i32` and `bool`** are allowed. No implicit casts.
4. **Arithmetic ops** (`+ - * / %`) require `i32` operands and yield `i32`.
5. **Comparison ops** (`== != < <= > >=`) require `i32` operands and yield `bool`.
6. **Logical ops** (`&& || !`) require `bool` operands and yield `bool`.
7. **`if` is an expression** and **must include `else`**. Both branches must return the same type.
8. **`while` is a statement** in this subset: use it only as `ExprStmt` (i.e., `while ... { ... };`).
9. **Block rule**: a `Block` must end with an expression (no trailing `;`). The value of the block
   is the value of its final expression.
10. **Function body**: a function’s block final expression must match the function return type.
11. **Call resolution**: all called functions are defined in the same program (no extern).
12. **No recursion limit changes, no macros, no modules, no structs/enums/traits**.

## Examples (valid)

```rust
fn add(x: i32, y: i32) -> i32 { x + y }

fn main() -> i32 {
    let a: i32 = 40;
    let b: i32 = if a > 0 { 2 } else { 0 };
    add(a, b)
}
```

```rust
fn main() -> i32 {
    let i: i32 = 0;
    while i < 10 { }
    0
}
```

## Examples (invalid in this subset)

```rust
fn main() -> i32 { let x: i32 = 1; x = 2; x } // reassignment
```

```rust
fn main() -> i32 { if true { 1 } else { false } } // branch types differ
```
