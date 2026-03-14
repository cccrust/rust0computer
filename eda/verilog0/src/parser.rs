/// Verilog Parser - produces an AST from a token stream

use crate::lexer::{Token, TokenKind};
use std::fmt;

// ============================================================
// AST Node Definitions
// ============================================================

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub ports: Vec<PortDecl>,
    pub items: Vec<ModuleItem>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct PortDecl {
    pub dir: PortDir,
    pub net_type: Option<NetType>,
    pub signed: bool,
    pub range: Option<Range>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetType {
    Wire,
    Reg,
    Integer,
    Real,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub msb: Expr,
    pub lsb: Expr,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    PortDecl(PortDecl),
    NetDecl(NetDecl),
    ParamDecl(ParamDecl),
    ContinuousAssign(ContAssign),
    AlwaysBlock(AlwaysBlock),
    InitialBlock(InitialBlock),
    ModuleInst(ModuleInst),
    GateInst(GateInst),
    GenerateBlock(GenerateBlock),
    TaskDecl(TaskDecl),
    FunctionDecl(FunctionDecl),
}

#[derive(Debug, Clone)]
pub struct NetDecl {
    pub net_type: NetType,
    pub signed: bool,
    pub range: Option<Range>,
    pub names: Vec<(String, Option<Expr>)>, // name, optional initializer
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub is_local: bool,
    pub range: Option<Range>,
    pub params: Vec<(String, Expr)>,
}

#[derive(Debug, Clone)]
pub struct ContAssign {
    pub lhs: Expr,
    pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct AlwaysBlock {
    pub sensitivity: Sensitivity,
    pub body: Stmt,
}

#[derive(Debug, Clone)]
pub struct InitialBlock {
    pub body: Stmt,
}

#[derive(Debug, Clone)]
pub enum Sensitivity {
    Star,                          // always @(*)
    List(Vec<SensitivityEvent>),   // always @(posedge clk, negedge rst)
    None,                          // always (no event control - combinational)
}

#[derive(Debug, Clone)]
pub struct SensitivityEvent {
    pub edge: Option<Edge>,
    pub signal: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edge {
    Posedge,
    Negedge,
}

#[derive(Debug, Clone)]
pub struct ModuleInst {
    pub module_name: String,
    pub params: Vec<ParamConnect>,
    pub instances: Vec<Instance>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub connections: Vec<PortConnect>,
}

#[derive(Debug, Clone)]
pub enum PortConnect {
    ByPosition(Expr),
    ByName(String, Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum ParamConnect {
    ByPosition(Expr),
    ByName(String, Expr),
}

#[derive(Debug, Clone)]
pub struct GateInst {
    pub gate_type: GateType,
    pub instances: Vec<GatePrimInst>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    Buf,
}

#[derive(Debug, Clone)]
pub struct GatePrimInst {
    pub name: Option<String>,
    pub terminals: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct GenerateBlock {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone)]
pub struct TaskDecl {
    pub name: String,
    pub ports: Vec<PortDecl>,
    pub body: Stmt,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub return_range: Option<Range>,
    pub ports: Vec<PortDecl>,
    pub body: Stmt,
}

// ============================================================
// Statements
// ============================================================

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),              // begin...end
    ForkJoin(Vec<Stmt>),           // fork...join
    If(IfStmt),
    Case(CaseStmt),
    For(ForStmt),
    While(WhileStmt),
    Repeat(RepeatStmt),
    Forever(Box<Stmt>),
    BlockingAssign(Expr, Expr),        // lhs = rhs
    NonBlockingAssign(Expr, Expr),     // lhs <= rhs
    EventControl(Sensitivity, Box<Stmt>),
    DelayControl(Expr, Box<Stmt>),
    SystemTask(String, Vec<Expr>),
    TaskCall(String, Vec<Expr>),
    Disable(String),
    Null,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_: Box<Stmt>,
    pub else_: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct CaseStmt {
    pub expr: Expr,
    pub kind: CaseKind,
    pub items: Vec<CaseItem>,
    pub default: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub enum CaseKind { Case, Casex, Casez }

#[derive(Debug, Clone)]
pub struct CaseItem {
    pub patterns: Vec<Expr>,
    pub stmt: Stmt,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Box<Stmt>,
    pub cond: Expr,
    pub step: Box<Stmt>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct RepeatStmt {
    pub count: Expr,
    pub body: Box<Stmt>,
}

// ============================================================
// Expressions
// ============================================================

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(u64),
    BasedLit { size: Option<u32>, base: char, value: String },
    RealLit(f64),
    StringLit(String),
    Ident(String),
    Index(Box<Expr>, Box<Expr>),            // arr[idx]
    Slice(Box<Expr>, Box<Expr>, Box<Expr>), // arr[msb:lsb]
    Member(Box<Expr>, String),              // obj.member
    Concat(Vec<Expr>),                      // {a, b, c}
    Repeat(Box<Expr>, Vec<Expr>),           // {n{a, b}}
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    FuncCall(String, Vec<Expr>),
    SystemCall(String, Vec<Expr>),
    HiZ,
    X,
    Z,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus, Minus, Not, BitNot,
    RedAnd, RedNand, RedOr, RedNor, RedXor, RedXnor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, CaseEq, CaseNe,
    Lt, Le, Gt, Ge,
    LogAnd, LogOr,
    BitAnd, BitOr, BitXor, BitXnor,
    Shl, Shr, AShl, AShr,
    Pow,
}

// ============================================================
// Parse Error
// ============================================================

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

// ============================================================
// Parser
// ============================================================

pub fn parse(tokens: Vec<Token>) -> Result<SourceFile, ParseError> {
    let mut p = Parser::new(tokens);
    p.parse_file()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        let t = self.peek().clone();
        if std::mem::discriminant(&t.kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(t)
        } else {
            Err(ParseError {
                msg: format!("Expected {:?}, got {:?}", kind, t.kind),
                line: t.line,
                col: t.col,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let t = self.peek().clone();
        match &t.kind {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(ParseError {
                msg: format!("Expected identifier, got {:?}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::EOF)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ---- Top Level ----

    fn parse_file(&mut self) -> Result<SourceFile, ParseError> {
        let mut modules = Vec::new();
        // Skip timescale/defines at top level
        while !self.at_eof() {
            match self.peek_kind().clone() {
                TokenKind::Timescale | TokenKind::Define | TokenKind::Include => {
                    self.advance();
                }
                TokenKind::Module => {
                    modules.push(self.parse_module()?);
                }
                TokenKind::EOF => break,
                _ => {
                    self.advance(); // skip unknown top-level tokens
                }
            }
        }
        Ok(SourceFile { modules })
    }

    fn parse_module(&mut self) -> Result<Module, ParseError> {
        let line = self.peek().line;
        self.expect(&TokenKind::Module)?;
        let name = self.expect_ident()?;

        let mut ports = Vec::new();

        // Port list: module foo(a, b, c) or ANSI style module foo(input a, output b)
        if self.eat(&TokenKind::LParen) {
            if !self.check(&TokenKind::RParen) {
                // ANSI or non-ANSI port list
                ports = self.parse_port_list()?;
            }
            self.expect(&TokenKind::RParen)?;
        }
        self.expect(&TokenKind::Semicolon)?;

        let mut items = Vec::new();
        while !self.check(&TokenKind::Endmodule) && !self.at_eof() {
            match self.parse_module_item() {
                Ok(item) => items.push(item),
                Err(e) => return Err(e),
            }
        }
        self.expect(&TokenKind::Endmodule)?;

        Ok(Module { name, ports, items, line })
    }

    fn parse_port_list(&mut self) -> Result<Vec<PortDecl>, ParseError> {
        let mut ports = Vec::new();
'port_loop: loop {
            // Check for ANSI-style port declarations
            let dir = match self.peek_kind().clone() {
                TokenKind::Input  => { self.advance(); Some(PortDir::Input) }
                TokenKind::Output => { self.advance(); Some(PortDir::Output) }
                TokenKind::Inout  => { self.advance(); Some(PortDir::Inout) }
                _ => None,
            };

            if let Some(dir) = dir {
                let net_type = self.parse_optional_net_type();
                let signed = self.eat(&TokenKind::Signed);
                let range = self.parse_optional_range()?;
                let mut names = Vec::new();
                names.push(self.expect_ident()?);
                while self.eat(&TokenKind::Comma) {
                    // Check if next item starts a new direction
                    if matches!(self.peek_kind(), TokenKind::Input | TokenKind::Output | TokenKind::Inout) {
                        // push current port and continue outer loop
                        ports.push(PortDecl { dir: dir.clone(), net_type: net_type.clone(), signed, range: range.clone(), names: names.clone() });
                        continue 'port_loop;
                    }
                    if self.check(&TokenKind::RParen) { break; }
                    names.push(self.expect_ident()?);
                }
                if !ports.last().map_or(false, |_| true) {
                    ports.push(PortDecl { dir, net_type, signed, range, names });
                }
            } else {
                // Non-ANSI: just identifier
                let name = self.expect_ident()?;
                ports.push(PortDecl {
                    dir: PortDir::Input, // placeholder
                    net_type: None,
                    signed: false,
                    range: None,
                    names: vec![name],
                });
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
            if self.check(&TokenKind::RParen) {
                break;
            }
        }
        Ok(ports)
    }

    fn parse_optional_net_type(&mut self) -> Option<NetType> {
        match self.peek_kind().clone() {
            TokenKind::Wire    => { self.advance(); Some(NetType::Wire) }
            TokenKind::Reg     => { self.advance(); Some(NetType::Reg) }
            TokenKind::Integer => { self.advance(); Some(NetType::Integer) }
            TokenKind::Real    => { self.advance(); Some(NetType::Real) }
            _ => None,
        }
    }

    fn parse_optional_range(&mut self) -> Result<Option<Range>, ParseError> {
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let msb = self.parse_expr()?;
            self.expect(&TokenKind::Colon)?;
            let lsb = self.parse_expr()?;
            self.expect(&TokenKind::RBracket)?;
            Ok(Some(Range { msb, lsb }))
        } else {
            Ok(None)
        }
    }

    fn parse_module_item(&mut self) -> Result<ModuleItem, ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Input | TokenKind::Output | TokenKind::Inout => {
                let dir = match self.advance().kind.clone() {
                    TokenKind::Input  => PortDir::Input,
                    TokenKind::Output => PortDir::Output,
                    _                 => PortDir::Inout,
                };
                let net_type = self.parse_optional_net_type();
                let signed = self.eat(&TokenKind::Signed);
                let range = self.parse_optional_range()?;
                let mut names = Vec::new();
                names.push(self.expect_ident()?);
                while self.eat(&TokenKind::Comma) {
                    names.push(self.expect_ident()?);
                }
                self.expect(&TokenKind::Semicolon)?;
                Ok(ModuleItem::PortDecl(PortDecl { dir, net_type, signed, range, names }))
            }

            TokenKind::Wire | TokenKind::Reg | TokenKind::Integer | TokenKind::Real => {
                let net_type = self.parse_optional_net_type().unwrap();
                let signed = self.eat(&TokenKind::Signed);
                let range = self.parse_optional_range()?;
                let mut names = Vec::new();
                loop {
                    let name = self.expect_ident()?;
                    let init = if self.eat(&TokenKind::Assign_) {
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    names.push((name, init));
                    if !self.eat(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::Semicolon)?;
                Ok(ModuleItem::NetDecl(NetDecl { net_type, signed, range, names }))
            }

            TokenKind::Parameter | TokenKind::Localparam => {
                let is_local = matches!(self.peek_kind(), TokenKind::Localparam);
                self.advance();
                let range = self.parse_optional_range()?;
                let mut params = Vec::new();
                loop {
                    let name = self.expect_ident()?;
                    self.expect(&TokenKind::Assign_)?;
                    let val = self.parse_expr()?;
                    params.push((name, val));
                    if !self.eat(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::Semicolon)?;
                Ok(ModuleItem::ParamDecl(ParamDecl { is_local, range, params }))
            }

            TokenKind::Assign => {
                self.advance();
                let lhs = self.parse_expr()?;
                self.expect(&TokenKind::Assign_)?;
                let rhs = self.parse_expr()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(ModuleItem::ContinuousAssign(ContAssign { lhs, rhs }))
            }

            TokenKind::Always => {
                self.advance();
                let sensitivity = self.parse_sensitivity()?;
                let body = self.parse_stmt()?;
                Ok(ModuleItem::AlwaysBlock(AlwaysBlock { sensitivity, body }))
            }

            TokenKind::Initial => {
                self.advance();
                let body = self.parse_stmt()?;
                Ok(ModuleItem::InitialBlock(InitialBlock { body }))
            }

            TokenKind::Generate => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenKind::Endgenerate) && !self.at_eof() {
                    items.push(self.parse_module_item()?);
                }
                self.expect(&TokenKind::Endgenerate)?;
                Ok(ModuleItem::GenerateBlock(GenerateBlock { items }))
            }

            TokenKind::And | TokenKind::Or | TokenKind::Not | TokenKind::Nand
            | TokenKind::Nor | TokenKind::Xor | TokenKind::Xnor | TokenKind::Buf => {
                self.parse_gate_inst()
            }

            TokenKind::Task => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Semicolon)?;
                let body = self.parse_stmt()?;
                self.expect(&TokenKind::Endtask)?;
                Ok(ModuleItem::TaskDecl(TaskDecl { name, ports: vec![], body }))
            }

            TokenKind::Function => {
                self.advance();
                let return_range = self.parse_optional_range()?;
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Semicolon)?;
                let body = self.parse_stmt()?;
                self.expect(&TokenKind::Endfunction)?;
                Ok(ModuleItem::FunctionDecl(FunctionDecl {
                    name, return_range, ports: vec![], body,
                }))
            }

            // Module instantiation or other identifier-starting items
            TokenKind::Ident(_) => {
                self.parse_module_instantiation()
            }

            TokenKind::Timescale | TokenKind::Semicolon => {
                self.advance();
                // null item
                self.parse_module_item()
            }

            _ => {
                Err(ParseError {
                    msg: format!("Unexpected token in module body: {:?}", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        }
    }

    fn parse_sensitivity(&mut self) -> Result<Sensitivity, ParseError> {
        if !self.check(&TokenKind::At) {
            return Ok(Sensitivity::None);
        }
        self.advance(); // consume @
        if self.eat(&TokenKind::Star) {
            return Ok(Sensitivity::Star);
        }
        // Check for @(*)
        if self.check(&TokenKind::LParen) {
            self.advance();
            if self.eat(&TokenKind::Star) {
                self.expect(&TokenKind::RParen)?;
                return Ok(Sensitivity::Star);
            }
            let mut events = Vec::new();
            loop {
                let edge = if self.eat(&TokenKind::Posedge) {
                    Some(Edge::Posedge)
                } else if self.eat(&TokenKind::Negedge) {
                    Some(Edge::Negedge)
                } else {
                    None
                };
                let signal = self.parse_expr()?;
                events.push(SensitivityEvent { edge, signal });
                if !self.eat(&TokenKind::Comma) && !self.eat(&TokenKind::Or) {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            Ok(Sensitivity::List(events))
        } else {
            // @signal (no parens)
            let edge = if self.eat(&TokenKind::Posedge) {
                Some(Edge::Posedge)
            } else if self.eat(&TokenKind::Negedge) {
                Some(Edge::Negedge)
            } else {
                None
            };
            let signal = self.parse_expr()?;
            Ok(Sensitivity::List(vec![SensitivityEvent { edge, signal }]))
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Begin => {
                self.advance();
                let mut stmts = Vec::new();
                while !self.check(&TokenKind::End) && !self.at_eof() {
                    stmts.push(self.parse_stmt()?);
                }
                self.expect(&TokenKind::End)?;
                Ok(Stmt::Block(stmts))
            }

            TokenKind::Fork => {
                self.advance();
                let mut stmts = Vec::new();
                while !self.check(&TokenKind::Join) && !self.at_eof() {
                    stmts.push(self.parse_stmt()?);
                }
                self.expect(&TokenKind::Join)?;
                Ok(Stmt::ForkJoin(stmts))
            }

            TokenKind::If => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                let then_ = Box::new(self.parse_stmt()?);
                let else_ = if self.eat(&TokenKind::Else) {
                    Some(Box::new(self.parse_stmt()?))
                } else {
                    None
                };
                Ok(Stmt::If(IfStmt { cond, then_, else_ }))
            }

            TokenKind::Case | TokenKind::Casex | TokenKind::Casez => {
                let kind = match self.advance().kind.clone() {
                    TokenKind::Case  => CaseKind::Case,
                    TokenKind::Casex => CaseKind::Casex,
                    _                => CaseKind::Casez,
                };
                self.expect(&TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                let mut items = Vec::new();
                let mut default = None;
                while !self.check(&TokenKind::Endcase) && !self.at_eof() {
                    if self.check(&TokenKind::Ident("default".to_string())) {
                        self.advance();
                        self.eat(&TokenKind::Colon);
                        default = Some(Box::new(self.parse_stmt()?));
                    } else {
                        let mut patterns = Vec::new();
                        patterns.push(self.parse_expr()?);
                        while self.eat(&TokenKind::Comma) {
                            patterns.push(self.parse_expr()?);
                        }
                        self.expect(&TokenKind::Colon)?;
                        let stmt = self.parse_stmt()?;
                        items.push(CaseItem { patterns, stmt });
                    }
                }
                self.expect(&TokenKind::Endcase)?;
                Ok(Stmt::Case(CaseStmt { expr, kind, items, default }))
            }

            TokenKind::For => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let init = Box::new(self.parse_stmt()?);
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::Semicolon)?;
                let step = Box::new(self.parse_stmt_no_semi()?);
                self.expect(&TokenKind::RParen)?;
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::For(ForStmt { init, cond, step, body }))
            }

            TokenKind::While => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::While(WhileStmt { cond, body }))
            }

            TokenKind::Repeat => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let count = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::Repeat(RepeatStmt { count, body }))
            }

            TokenKind::Forever => {
                self.advance();
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::Forever(body))
            }

            TokenKind::At => {
                let sens = self.parse_sensitivity()?;
                let stmt = Box::new(self.parse_stmt()?);
                Ok(Stmt::EventControl(sens, stmt))
            }

            TokenKind::Hash => {
                self.advance();
                let delay = self.parse_primary()?;
                let stmt = Box::new(self.parse_stmt()?);
                Ok(Stmt::DelayControl(delay, stmt))
            }

            TokenKind::SystemTask(s) => {
                let s = s.clone();
                self.advance();
                let args = if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.eat(&TokenKind::Comma) {
                            if self.check(&TokenKind::RParen) { break; }
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    args
                } else {
                    Vec::new()
                };
                self.expect(&TokenKind::Semicolon)?;
                Ok(Stmt::SystemTask(s, args))
            }

            TokenKind::Disable => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Stmt::Disable(name))
            }

            TokenKind::Semicolon => {
                self.advance();
                Ok(Stmt::Null)
            }

            // Assignment or task call
            _ => {
                let lhs = self.parse_expr()?;
                match self.peek_kind().clone() {
                    TokenKind::Assign_ => {
                        self.advance();
                        let rhs = self.parse_expr()?;
                        self.expect(&TokenKind::Semicolon)?;
                        Ok(Stmt::BlockingAssign(lhs, rhs))
                    }
                    TokenKind::NonBlockAssign => {
                        self.advance();
                        let rhs = self.parse_expr()?;
                        self.expect(&TokenKind::Semicolon)?;
                        Ok(Stmt::NonBlockingAssign(lhs, rhs))
                    }
                    TokenKind::Semicolon => {
                        self.advance();
                        // Task call (expression used as statement)
                        if let Expr::Ident(name) = lhs {
                            Ok(Stmt::TaskCall(name, vec![]))
                        } else {
                            Ok(Stmt::Null)
                        }
                    }
                    _ => {
                        Err(ParseError {
                            msg: format!("Expected assignment or ';', got {:?}", self.peek_kind()),
                            line: self.peek().line,
                            col: self.peek().col,
                        })
                    }
                }
            }
        }
    }

    /// Parse a statement without consuming a trailing semicolon (used in for-loop step)
    fn parse_stmt_no_semi(&mut self) -> Result<Stmt, ParseError> {
        let lhs = self.parse_expr()?;
        match self.peek_kind().clone() {
            TokenKind::Assign_ => {
                self.advance();
                let rhs = self.parse_expr()?;
                Ok(Stmt::BlockingAssign(lhs, rhs))
            }
            TokenKind::NonBlockAssign => {
                self.advance();
                let rhs = self.parse_expr()?;
                Ok(Stmt::NonBlockingAssign(lhs, rhs))
            }
            _ => Ok(Stmt::Null),
        }
    }

    // ---- Module Instantiation ----

    fn parse_module_instantiation(&mut self) -> Result<ModuleItem, ParseError> {
        let module_name = self.expect_ident()?;

        // Optional parameter override: #(.PARAM(val), ...)
        let mut params = Vec::new();
        if self.eat(&TokenKind::Hash) {
            if self.eat(&TokenKind::LParen) {
                while !self.check(&TokenKind::RParen) && !self.at_eof() {
                    if self.eat(&TokenKind::Dot) {
                        let pname = self.expect_ident()?;
                        self.expect(&TokenKind::LParen)?;
                        let val = self.parse_expr()?;
                        self.expect(&TokenKind::RParen)?;
                        params.push(ParamConnect::ByName(pname, val));
                    } else {
                        params.push(ParamConnect::ByPosition(self.parse_expr()?));
                    }
                    if !self.eat(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::RParen)?;
            }
        }

        let mut instances = Vec::new();
        loop {
            let inst_name = self.expect_ident()?;
            self.expect(&TokenKind::LParen)?;
            let mut connections = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.at_eof() {
                if self.eat(&TokenKind::Dot) {
                    let port_name = self.expect_ident()?;
                    self.expect(&TokenKind::LParen)?;
                    let expr = if self.check(&TokenKind::RParen) {
                        None
                    } else {
                        Some(self.parse_expr()?)
                    };
                    self.expect(&TokenKind::RParen)?;
                    connections.push(PortConnect::ByName(port_name, expr));
                } else {
                    connections.push(PortConnect::ByPosition(self.parse_expr()?));
                }
                if !self.eat(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RParen)?;
            instances.push(Instance { name: inst_name, connections });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(ModuleItem::ModuleInst(ModuleInst { module_name, params, instances }))
    }

    fn parse_gate_inst(&mut self) -> Result<ModuleItem, ParseError> {
        let gate_type = match self.advance().kind.clone() {
            TokenKind::And  => GateType::And,
            TokenKind::Or   => GateType::Or,
            TokenKind::Not  => GateType::Not,
            TokenKind::Nand => GateType::Nand,
            TokenKind::Nor  => GateType::Nor,
            TokenKind::Xor  => GateType::Xor,
            TokenKind::Xnor => GateType::Xnor,
            TokenKind::Buf  => GateType::Buf,
            _ => unreachable!(),
        };
        let mut instances = Vec::new();
        loop {
            let name = if let TokenKind::Ident(_) = self.peek_kind() {
                if self.tokens.get(self.pos + 1).map_or(false, |t| matches!(t.kind, TokenKind::LParen)) {
                    Some(self.expect_ident()?)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&TokenKind::LParen)?;
            let mut terminals = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.at_eof() {
                terminals.push(self.parse_expr()?);
                if !self.eat(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RParen)?;
            instances.push(GatePrimInst { name, terminals });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(ModuleItem::GateInst(GateInst { gate_type, instances }))
    }

    // ---- Expression Parsing (Pratt/precedence-climbing) ----

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr, ParseError> {
        let cond = self.parse_binary(0)?;
        if self.eat(&TokenKind::Ternary) {
            let then_ = self.parse_expr()?;
            self.expect(&TokenKind::Colon)?;
            let else_ = self.parse_expr()?;
            Ok(Expr::Ternary(Box::new(cond), Box::new(then_), Box::new(else_)))
        } else {
            Ok(cond)
        }
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let (prec, right_assoc) = match self.peek_kind() {
                TokenKind::LogOr  => (2, false),
                TokenKind::LogAnd => (3, false),
                TokenKind::BitOr  => (4, false),
                TokenKind::BitXor | TokenKind::BitXnor => (5, false),
                TokenKind::BitAnd => (6, false),
                TokenKind::Eq | TokenKind::Ne | TokenKind::CaseEq | TokenKind::CaseNe => (7, false),
                TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge => (8, false),
                TokenKind::Shl | TokenKind::Shr | TokenKind::AShl | TokenKind::AShr => (9, false),
                TokenKind::Plus | TokenKind::Minus => (10, false),
                TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (11, false),
                _ => break,
            };
            if prec < min_prec { break; }
            let op = self.advance().kind.clone();
            let next_prec = if right_assoc { prec } else { prec + 1 };
            let rhs = self.parse_binary(next_prec)?;
            let bin_op = match op {
                TokenKind::Plus     => BinaryOp::Add,
                TokenKind::Minus    => BinaryOp::Sub,
                TokenKind::Star     => BinaryOp::Mul,
                TokenKind::Slash    => BinaryOp::Div,
                TokenKind::Percent  => BinaryOp::Mod,
                TokenKind::Eq       => BinaryOp::Eq,
                TokenKind::Ne       => BinaryOp::Ne,
                TokenKind::CaseEq   => BinaryOp::CaseEq,
                TokenKind::CaseNe   => BinaryOp::CaseNe,
                TokenKind::Lt       => BinaryOp::Lt,
                TokenKind::NonBlockAssign => BinaryOp::Le, // <= in expression context
                TokenKind::Gt       => BinaryOp::Gt,
                TokenKind::Ge       => BinaryOp::Ge,
                TokenKind::LogAnd   => BinaryOp::LogAnd,
                TokenKind::LogOr    => BinaryOp::LogOr,
                TokenKind::BitAnd   => BinaryOp::BitAnd,
                TokenKind::BitOr    => BinaryOp::BitOr,
                TokenKind::BitXor   => BinaryOp::BitXor,
                TokenKind::BitXnor  => BinaryOp::BitXnor,
                TokenKind::Shl      => BinaryOp::Shl,
                TokenKind::Shr      => BinaryOp::Shr,
                TokenKind::AShl     => BinaryOp::AShl,
                TokenKind::AShr     => BinaryOp::AShr,
                _ => BinaryOp::Add, // fallback
            };
            lhs = Expr::Binary(bin_op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let op = match self.peek_kind().clone() {
            TokenKind::Plus   => Some(UnaryOp::Plus),
            TokenKind::Minus  => Some(UnaryOp::Minus),
            TokenKind::LogNot => Some(UnaryOp::Not),
            TokenKind::BitNot => Some(UnaryOp::BitNot),
            TokenKind::BitAnd => Some(UnaryOp::RedAnd),
            TokenKind::BitOr  => Some(UnaryOp::RedOr),
            TokenKind::BitXor => Some(UnaryOp::RedXor),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let expr = self.parse_postfix()?;
            Ok(Expr::Unary(op, Box::new(expr)))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenKind::LBracket) {
                self.advance();
                let idx = self.parse_expr()?;
                if self.eat(&TokenKind::Colon) {
                    let lsb = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Slice(Box::new(expr), Box::new(idx), Box::new(lsb));
                } else {
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Index(Box::new(expr), Box::new(idx));
                }
            } else if self.eat(&TokenKind::Dot) {
                let member = self.expect_ident()?;
                expr = Expr::Member(Box::new(expr), member);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::IntLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLit(n))
            }
            TokenKind::BasedLiteral { size, base, value } => {
                let size = *size;
                let base = *base;
                let value = value.clone();
                self.advance();
                Ok(Expr::BasedLit { size, base, value })
            }
            TokenKind::RealLiteral(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::RealLit(f))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLit(s))
            }
            TokenKind::SystemTask(s) => {
                let s = s.clone();
                self.advance();
                let args = if self.eat(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.at_eof() {
                        args.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    args
                } else { Vec::new() };
                Ok(Expr::SystemCall(s, args))
            }
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                // Function call?
                if self.eat(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.at_eof() {
                        args.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::FuncCall(s, args))
                } else {
                    Ok(Expr::Ident(s))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                self.advance();
                // Repetition: {n{...}} or concatenation: {a, b, c}
                let first = self.parse_expr()?;
                if self.eat(&TokenKind::LBrace) {
                    // Replication
                    let mut parts = Vec::new();
                    parts.push(self.parse_expr()?);
                    while self.eat(&TokenKind::Comma) {
                        parts.push(self.parse_expr()?);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Repeat(Box::new(first), parts))
                } else if self.eat(&TokenKind::Comma) {
                    let mut parts = vec![first];
                    parts.push(self.parse_expr()?);
                    while self.eat(&TokenKind::Comma) {
                        parts.push(self.parse_expr()?);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Concat(parts))
                } else {
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Concat(vec![first]))
                }
            }
            _ => Err(ParseError {
                msg: format!("Unexpected token in expression: {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            }),
        }
    }
}
