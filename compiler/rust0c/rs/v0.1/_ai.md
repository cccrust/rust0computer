
## ccc: 請寫出目前支援的語法規格（EBNF)

這題很棒，我把目前「實際支援」的語法整理成 EBNF（對應我們現在的 lexer/parser/sema/codegen 行為）。

**語法規格（EBNF）**

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

**語意限制（重要）**
- `if` 是「表達式」，必須有 `else`
- `if` 兩邊的 `BlockBody` 必須回傳同型別
- `let` 只有宣告（不能賦值更新）
- 支援 `i32` 和 `bool`
- 變數遮蔽（同名 `let`）目前允許，但會以最內層為主

如果你希望我把這份規格也整理成 `compiler/SYNTAX.md` 文件，我可以直接補上。