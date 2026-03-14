# Compiler Supported Syntax (EBNF)

This document describes the Rust subset currently supported by the compiler.

```ebnf
Program     = { Function } ;

Function    = "fn", Ident, "(", [ ParamList ], ")", "->", Type, "{", BlockBody, "}" ;

ParamList   = Param, { ",", Param } ;
Param       = Ident, ":", Type ;

Type        = "i32" | "bool" ;

BlockBody   = { LetStmt }, Expr ;

LetStmt     = "let", Ident, ":", Type, "=", Expr, ";" ;

Expr        = IfExpr | CmpExpr ;

IfExpr      = "if", Expr, "{", BlockBody, "}", "else", "{", BlockBody, "}" ;

CmpExpr     = AddExpr, { CmpOp, AddExpr } ;
CmpOp       = "==" | "!=" | "<" | "<=" | ">" | ">=" ;

AddExpr     = MulExpr, { ("+" | "-"), MulExpr } ;

MulExpr     = Primary, { "*", Primary } ;

Primary     = Int
            | "true"
            | "false"
            | Ident [ "(", [ ArgList ], ")" ]
            | "(", Expr, ")"
            | "{", BlockBody, "}" ;

ArgList     = Expr, { ",", Expr } ;

Ident       = Letter, { Letter | Digit | "_" } ;
Int         = Digit, { Digit } ;
Letter      = "A".."Z" | "a".."z" | "_" ;
Digit       = "0".."9" ;
```

## Semantic Notes
- `if` is an expression and must include `else`.
- The `then` and `else` blocks must return the same type.
- `let` is declaration-only (no reassignment yet).
- Supported types: `i32`, `bool`.
- Shadowing is allowed; the innermost binding takes precedence.
