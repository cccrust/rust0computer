use crate::ast::{BinOp, Expr, Function, Param, Program, Stmt, Type, UnaryOp};
use crate::lexer::Token;
use anyhow::{bail, Result};

pub fn parse(tokens: &[Token]) -> Result<Program> {
    let mut p = Parser { tokens, pos: 0 };
    let mut functions = Vec::new();
    while !p.eof() {
        functions.push(p.parse_function()?);
    }
    Ok(Program { functions })
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<()> {
        let tok = self.next().ok_or_else(|| anyhow::anyhow!("unexpected eof"))?;
        if tok == expected {
            Ok(())
        } else {
            bail!("expected {:?}, got {:?}", expected, tok)
        }
    }

    fn parse_function(&mut self) -> Result<Function> {
        self.expect(&Token::Fn)?;
        let name = match self.next() {
            Some(Token::Ident(s)) => s.clone(),
            other => bail!("expected function name, got {:?}", other),
        };
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Arrow)?;
        let return_type = self.parse_type()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_block_body()?;
        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(params);
        }
        loop {
            let name = match self.next() {
                Some(Token::Ident(s)) => s.clone(),
                other => bail!("expected param name, got {:?}", other),
            };
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });
            if matches!(self.peek(), Some(Token::Comma)) {
                self.next();
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        match self.next() {
            Some(Token::I32) => Ok(Type::I32),
            Some(Token::Bool) => Ok(Type::Bool),
            other => bail!("expected type, got {:?}", other),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        if matches!(self.peek(), Some(Token::If)) {
            return self.parse_if();
        }
        if matches!(self.peek(), Some(Token::While)) {
            return self.parse_while();
        }
        self.parse_or()
    }

    fn parse_if(&mut self) -> Result<Expr> {
        self.expect(&Token::If)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let then_expr = self.parse_block_body()?;
        self.expect(&Token::Else)?;
        self.expect(&Token::LBrace)?;
        let else_expr = self.parse_block_body()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    fn parse_while(&mut self) -> Result<Expr> {
        self.expect(&Token::While)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_block_body()?;
        Ok(Expr::While {
            cond: Box::new(cond),
            body: Box::new(body),
        })
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;
        loop {
            if !matches!(self.peek(), Some(Token::OrOr)) {
                break;
            }
            self.next();
            let rhs = self.parse_and()?;
            expr = Expr::Binary {
                op: BinOp::Or,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_eq()?;
        loop {
            if !matches!(self.peek(), Some(Token::AndAnd)) {
                break;
            }
            self.next();
            let rhs = self.parse_eq()?;
            expr = Expr::Binary {
                op: BinOp::And,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_eq(&mut self) -> Result<Expr> {
        let mut expr = self.parse_cmp()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::NotEq) => BinOp::Ne,
                _ => break,
            };
            self.next();
            let rhs = self.parse_cmp()?;
            expr = Expr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_cmp(&mut self) -> Result<Expr> {
        let mut expr = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::Le) => BinOp::Le,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::Ge) => BinOp::Ge,
                _ => break,
            };
            self.next();
            let rhs = self.parse_add()?;
            expr = Expr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut expr = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.next();
            let rhs = self.parse_mul()?;
            expr = Expr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Rem,
                _ => break,
            };
            self.next();
            let rhs = self.parse_unary()?;
            expr = Expr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(Token::Bang) => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Minus) => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.next() {
            Some(Token::Int(v)) => Ok(Expr::Int(*v)),
            Some(Token::True) => Ok(Expr::Bool(true)),
            Some(Token::False) => Ok(Expr::Bool(false)),
            Some(Token::Ident(name)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call {
                        name: name.clone(),
                        args,
                    })
                } else {
                    Ok(Expr::Ident(name.clone()))
                }
            }
            Some(Token::LBrace) => self.parse_block_body(),
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            other => bail!("unexpected token in primary: {:?}", other),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }
        loop {
            let expr = self.parse_expr()?;
            args.push(expr);
            if matches!(self.peek(), Some(Token::Comma)) {
                self.next();
                continue;
            }
            break;
        }
        Ok(args)
    }

    fn parse_block_body(&mut self) -> Result<Expr> {
        let mut stmts = Vec::new();
        loop {
            if matches!(self.peek(), Some(Token::Let)) {
                stmts.push(self.parse_let_stmt()?);
                continue;
            }
            let expr = self.parse_expr()?;
            if matches!(self.peek(), Some(Token::Semicolon)) {
                self.next();
                stmts.push(Stmt::Expr(expr));
                continue;
            }
            self.expect(&Token::RBrace)?;
            return Ok(Expr::Block {
                stmts,
                expr: Box::new(expr),
            });
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::Let)?;
        let name = match self.next() {
            Some(Token::Ident(s)) => s.clone(),
            other => bail!("expected let name, got {:?}", other),
        };
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Let { name, ty, value })
    }
}
