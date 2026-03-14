/// Verilog Lexer
/// Tokenizes Verilog source code into a stream of tokens.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Module,
    Endmodule,
    Input,
    Output,
    Inout,
    Wire,
    Reg,
    Integer,
    Real,
    Parameter,
    Localparam,
    Always,
    Initial,
    Begin,
    End,
    If,
    Else,
    Case,
    Casex,
    Casez,
    Endcase,
    For,
    While,
    Repeat,
    Forever,
    Assign,
    Posedge,
    Negedge,
    Or,
    And,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    Buf,
    Defparam,
    Task,
    Endtask,
    Function,
    Endfunction,
    Disable,
    Fork,
    Join,
    Generate,
    Endgenerate,
    Genvar,
    Signed,
    Unsigned,

    // System tasks
    SystemTask(String),  // $display, $monitor, $finish, etc.

    // Identifiers and literals
    Ident(String),
    IntLiteral(u64),
    BasedLiteral { size: Option<u32>, base: char, value: String },
    StringLiteral(String),
    RealLiteral(f64),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,          // ==
    Ne,          // !=
    CaseEq,      // ===
    CaseNe,      // !==
    Lt,          // <
    Le,          // <=
    Gt,          // >
    Ge,          // >=
    LogAnd,      // &&
    LogOr,       // ||
    LogNot,      // !
    BitAnd,      // &
    BitOr,       // |
    BitXor,      // ^
    BitXnor,     // ~^  or ^~
    BitNot,      // ~
    Shl,         // <<
    Shr,         // >>
    AShl,        // <<<
    AShr,        // >>>
    Ternary,     // ?
    Colon,       // :
    Semicolon,   // ;
    Comma,       // ,
    Dot,         // .
    Hash,        // #
    At,          // @
    Dollar,      // $
    Assign_,     // =  (blocking assign / continuous)
    NonBlockAssign, // <=
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Timing
    Delay,       // # followed by number (treated as Hash + number)

    // Preprocessor
    Timescale,
    Define,
    Include,
    Ifdef,
    Ifndef,
    Else_,
    Endif,

    EOF,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lex error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Lexer {
            src: source.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.src.get(self.pos).copied()?;
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            // Skip whitespace
            while self.peek().map_or(false, |c| c.is_ascii_whitespace()) {
                self.advance();
            }
            // Single-line comment
            if self.peek() == Some(b'/') && self.peek2() == Some(b'/') {
                while self.peek().map_or(false, |c| c != b'\n') {
                    self.advance();
                }
                continue;
            }
            // Block comment
            if self.peek() == Some(b'/') && self.peek2() == Some(b'*') {
                self.advance(); self.advance();
                loop {
                    if self.peek().is_none() {
                        return Err(LexError {
                            msg: "Unterminated block comment".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    if self.peek() == Some(b'*') && self.peek2() == Some(b'/') {
                        self.advance(); self.advance();
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                s.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn keyword_or_ident(&self, s: &str) -> TokenKind {
        match s {
            "module"      => TokenKind::Module,
            "endmodule"   => TokenKind::Endmodule,
            "input"       => TokenKind::Input,
            "output"      => TokenKind::Output,
            "inout"       => TokenKind::Inout,
            "wire"        => TokenKind::Wire,
            "reg"         => TokenKind::Reg,
            "integer"     => TokenKind::Integer,
            "real"        => TokenKind::Real,
            "parameter"   => TokenKind::Parameter,
            "localparam"  => TokenKind::Localparam,
            "always"      => TokenKind::Always,
            "initial"     => TokenKind::Initial,
            "begin"       => TokenKind::Begin,
            "end"         => TokenKind::End,
            "if"          => TokenKind::If,
            "else"        => TokenKind::Else,
            "case"        => TokenKind::Case,
            "casex"       => TokenKind::Casex,
            "casez"       => TokenKind::Casez,
            "endcase"     => TokenKind::Endcase,
            "for"         => TokenKind::For,
            "while"       => TokenKind::While,
            "repeat"      => TokenKind::Repeat,
            "forever"     => TokenKind::Forever,
            "assign"      => TokenKind::Assign,
            "posedge"     => TokenKind::Posedge,
            "negedge"     => TokenKind::Negedge,
            "or"          => TokenKind::Or,
            "and"         => TokenKind::And,
            "not"         => TokenKind::Not,
            "nand"        => TokenKind::Nand,
            "nor"         => TokenKind::Nor,
            "xor"         => TokenKind::Xor,
            "xnor"        => TokenKind::Xnor,
            "buf"         => TokenKind::Buf,
            "defparam"    => TokenKind::Defparam,
            "task"        => TokenKind::Task,
            "endtask"     => TokenKind::Endtask,
            "function"    => TokenKind::Function,
            "endfunction" => TokenKind::Endfunction,
            "disable"     => TokenKind::Disable,
            "fork"        => TokenKind::Fork,
            "join"        => TokenKind::Join,
            "generate"    => TokenKind::Generate,
            "endgenerate" => TokenKind::Endgenerate,
            "genvar"      => TokenKind::Genvar,
            "signed"      => TokenKind::Signed,
            "unsigned"    => TokenKind::Unsigned,
            _             => TokenKind::Ident(s.to_string()),
        }
    }

    fn read_number(&mut self) -> Result<TokenKind, LexError> {
        let mut num_str = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == b'_' {
                num_str.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        // Check for based literal: 8'b1010, 4'hF, etc.
        if self.peek() == Some(b'\'') {
            let size: u32 = num_str.replace('_', "").parse().unwrap_or(0);
            self.advance(); // consume '
            let base = match self.peek() {
                Some(b'b') | Some(b'B') => { self.advance(); 'b' }
                Some(b'o') | Some(b'O') => { self.advance(); 'o' }
                Some(b'd') | Some(b'D') => { self.advance(); 'd' }
                Some(b'h') | Some(b'H') => { self.advance(); 'h' }
                _ => return Err(LexError {
                    msg: "Expected base specifier (b/o/d/h) after '".to_string(),
                    line: self.line, col: self.col,
                }),
            };
            let mut value = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'x' || c == b'z'
                    || c == b'X' || c == b'Z'
                {
                    value.push(c as char);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(TokenKind::BasedLiteral {
                size: if size > 0 { Some(size) } else { None },
                base,
                value,
            });
        }
        // Real number?
        if self.peek() == Some(b'.') && self.peek2().map_or(false, |c| c.is_ascii_digit()) {
            num_str.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == b'_' {
                    num_str.push(c as char);
                    self.advance();
                } else {
                    break;
                }
            }
            let val: f64 = num_str.replace('_', "").parse().unwrap_or(0.0);
            return Ok(TokenKind::RealLiteral(val));
        }
        let val: u64 = num_str.replace('_', "").parse().unwrap_or(0);
        Ok(TokenKind::IntLiteral(val))
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        let mut s = String::new();
        self.advance(); // consume opening "
        loop {
            match self.peek() {
                None => return Err(LexError {
                    msg: "Unterminated string literal".to_string(),
                    line: self.line, col: self.col,
                }),
                Some(b'"') => { self.advance(); break; }
                Some(b'\\') => {
                    self.advance();
                    match self.advance() {
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        Some(c)  => { s.push('\\'); s.push(c as char); }
                        None     => break,
                    }
                }
                Some(c) => { s.push(c as char); self.advance(); }
            }
        }
        Ok(s)
    }

    fn read_system_task(&mut self) -> TokenKind {
        let mut s = String::from("$");
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' {
                s.push(c as char);
                self.advance();
            } else {
                break;
            }
        }
        TokenKind::SystemTask(s)
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;

            let line = self.line;
            let col = self.col;

            let ch = match self.peek() {
                None => {
                    tokens.push(Token { kind: TokenKind::EOF, text: "".to_string(), line, col });
                    break;
                }
                Some(c) => c,
            };

            // Preprocessor directives
            if ch == b'`' {
                self.advance();
                let directive = self.read_ident();
                let kind = match directive.as_str() {
                    "timescale" => {
                        // consume rest of line
                        while self.peek().map_or(false, |c| c != b'\n') {
                            self.advance();
                        }
                        TokenKind::Timescale
                    }
                    "define"  => TokenKind::Define,
                    "include" => TokenKind::Include,
                    "ifdef"   => TokenKind::Ifdef,
                    "ifndef"  => TokenKind::Ifndef,
                    "else"    => TokenKind::Else_,
                    "endif"   => TokenKind::Endif,
                    _         => TokenKind::Ident(format!("`{}", directive)),
                };
                tokens.push(Token { kind, text: directive, line, col });
                continue;
            }

            // Identifiers / keywords
            if ch.is_ascii_alphabetic() || ch == b'_' {
                let ident = self.read_ident();
                let kind = self.keyword_or_ident(&ident);
                tokens.push(Token { kind, text: ident, line, col });
                continue;
            }

            // Escaped identifiers: \identifier
            if ch == b'\\' {
                self.advance();
                let mut ident = String::from("\\");
                while self.peek().map_or(false, |c| !c.is_ascii_whitespace()) {
                    ident.push(self.advance().unwrap() as char);
                }
                tokens.push(Token {
                    kind: TokenKind::Ident(ident.clone()),
                    text: ident,
                    line, col,
                });
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() {
                let kind = self.read_number()?;
                let text = format!("{:?}", kind);
                tokens.push(Token { kind, text, line, col });
                continue;
            }

            // Based literal starting with ' (e.g., 'b1, 'h...)
            if ch == b'\'' {
                self.advance();
                let base = match self.peek() {
                    Some(b'b') | Some(b'B') => { self.advance(); 'b' }
                    Some(b'o') | Some(b'O') => { self.advance(); 'o' }
                    Some(b'd') | Some(b'D') => { self.advance(); 'd' }
                    Some(b'h') | Some(b'H') => { self.advance(); 'h' }
                    Some(b's') | Some(b'S') => { self.advance(); 's' } // signed
                    _ => {
                        tokens.push(Token { kind: TokenKind::Colon, text: "'".to_string(), line, col });
                        continue;
                    }
                };
                let mut value = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'x' || c == b'z' {
                        value.push(c as char);
                        self.advance();
                    } else { break; }
                }
                tokens.push(Token {
                    kind: TokenKind::BasedLiteral { size: None, base, value: value.clone() },
                    text: value, line, col,
                });
                continue;
            }

            // String literals
            if ch == b'"' {
                let s = self.read_string()?;
                tokens.push(Token {
                    kind: TokenKind::StringLiteral(s.clone()),
                    text: s, line, col,
                });
                continue;
            }

            // System tasks
            if ch == b'$' {
                self.advance();
                // check if next char is alphanumeric (system task) or just dollar
                if self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == b'_') {
                    let mut s = String::from("$");
                    while let Some(c) = self.peek() {
                        if c.is_ascii_alphanumeric() || c == b'_' {
                            s.push(c as char);
                            self.advance();
                        } else { break; }
                    }
                    tokens.push(Token {
                        kind: TokenKind::SystemTask(s.clone()),
                        text: s, line, col,
                    });
                } else {
                    tokens.push(Token { kind: TokenKind::Dollar, text: "$".to_string(), line, col });
                }
                continue;
            }

            // Operators and punctuation
            self.advance();
            let kind = match ch {
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Minus,
                b'*' => {
                    if self.peek() == Some(b'*') { self.advance(); TokenKind::Star } // ** (power)
                    else { TokenKind::Star }
                }
                b'/' => TokenKind::Slash,
                b'%' => TokenKind::Percent,
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b'[' => TokenKind::LBracket,
                b']' => TokenKind::RBracket,
                b'{' => TokenKind::LBrace,
                b'}' => TokenKind::RBrace,
                b';' => TokenKind::Semicolon,
                b',' => TokenKind::Comma,
                b'.' => TokenKind::Dot,
                b'#' => TokenKind::Hash,
                b'@' => TokenKind::At,
                b'?' => TokenKind::Ternary,
                b':' => TokenKind::Colon,
                b'=' => {
                    if self.peek() == Some(b'=') {
                        self.advance();
                        if self.peek() == Some(b'=') { self.advance(); TokenKind::CaseEq }
                        else { TokenKind::Eq }
                    } else {
                        TokenKind::Assign_
                    }
                }
                b'!' => {
                    if self.peek() == Some(b'=') {
                        self.advance();
                        if self.peek() == Some(b'=') { self.advance(); TokenKind::CaseNe }
                        else { TokenKind::Ne }
                    } else {
                        TokenKind::LogNot
                    }
                }
                b'<' => {
                    if self.peek() == Some(b'=') {
                        self.advance();
                        TokenKind::NonBlockAssign  // <= (also Le, context-dependent)
                    } else if self.peek() == Some(b'<') {
                        self.advance();
                        if self.peek() == Some(b'<') { self.advance(); TokenKind::AShl }
                        else { TokenKind::Shl }
                    } else {
                        TokenKind::Lt
                    }
                }
                b'>' => {
                    if self.peek() == Some(b'=') { self.advance(); TokenKind::Ge }
                    else if self.peek() == Some(b'>') {
                        self.advance();
                        if self.peek() == Some(b'>') { self.advance(); TokenKind::AShr }
                        else { TokenKind::Shr }
                    } else {
                        TokenKind::Gt
                    }
                }
                b'&' => {
                    if self.peek() == Some(b'&') { self.advance(); TokenKind::LogAnd }
                    else { TokenKind::BitAnd }
                }
                b'|' => {
                    if self.peek() == Some(b'|') { self.advance(); TokenKind::LogOr }
                    else { TokenKind::BitOr }
                }
                b'^' => {
                    if self.peek() == Some(b'~') { self.advance(); TokenKind::BitXnor }
                    else { TokenKind::BitXor }
                }
                b'~' => {
                    if self.peek() == Some(b'^') { self.advance(); TokenKind::BitXnor }
                    else if self.peek() == Some(b'&') { self.advance(); TokenKind::Nand.into_bitnot_variant() }
                    else if self.peek() == Some(b'|') { self.advance(); TokenKind::Nor.into_bitnot_variant() }
                    else { TokenKind::BitNot }
                }
                _ => {
                    return Err(LexError {
                        msg: format!("Unexpected character: '{}'", ch as char),
                        line, col,
                    });
                }
            };

            tokens.push(Token {
                kind,
                text: (ch as char).to_string(),
                line,
                col,
            });
        }

        Ok(tokens)
    }
}

// Helper trait for ~& (reduction nand) and ~| (reduction nor)
trait IntoVariant {
    fn into_bitnot_variant(self) -> TokenKind;
}
impl IntoVariant for TokenKind {
    fn into_bitnot_variant(self) -> TokenKind {
        self // return as-is (reduction operators use same tokens)
    }
}
