use crate::ast::{BinOp, Expr, Function, Program, Stmt, Type, UnaryOp};
use anyhow::{bail, Result};
use std::collections::HashMap;

pub fn analyze(program: Program) -> Result<Program> {
    let mut fun_types = HashMap::new();
    for f in &program.functions {
        fun_types.insert(
            f.name.clone(),
            (f.params.iter().map(|p| p.ty.clone()).collect::<Vec<_>>(), f.return_type.clone()),
        );
    }

    for f in &program.functions {
        check_function(f, &fun_types)?;
    }

    Ok(program)
}

fn check_function(
    f: &Function,
    fun_types: &HashMap<String, (Vec<Type>, Type)>,
) -> Result<()> {
    let mut locals = HashMap::new();
    for p in &f.params {
        locals.insert(p.name.clone(), p.ty.clone());
    }
    let ty = check_expr(&f.body, &locals, fun_types)?;
    if ty != f.return_type {
        bail!(
            "function {} returns {:?} but body is {:?}",
            f.name,
            f.return_type,
            ty
        );
    }
    Ok(())
}

fn check_expr(
    e: &Expr,
    locals: &HashMap<String, Type>,
    fun_types: &HashMap<String, (Vec<Type>, Type)>,
) -> Result<Type> {
    match e {
        Expr::Int(_) => Ok(Type::I32),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Ident(name) => locals
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown identifier {}", name)),
        Expr::Unary { op, expr } => {
            let t = check_expr(expr, locals, fun_types)?;
            match op {
                UnaryOp::Neg => {
                    if t != Type::I32 {
                        bail!("unary '-' expects i32");
                    }
                    Ok(Type::I32)
                }
                UnaryOp::Not => {
                    if t != Type::Bool {
                        bail!("unary '!' expects bool");
                    }
                    Ok(Type::Bool)
                }
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let l = check_expr(lhs, locals, fun_types)?;
            let r = check_expr(rhs, locals, fun_types)?;
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                    if l != Type::I32 || r != Type::I32 {
                        bail!("arithmetic expects i32");
                    }
                    Ok(Type::I32)
                }
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    if l != Type::I32 || r != Type::I32 {
                        bail!("comparison expects i32");
                    }
                    Ok(Type::Bool)
                }
                BinOp::Eq | BinOp::Ne => {
                    if l != r {
                        bail!("equality expects same types");
                    }
                    Ok(Type::Bool)
                }
                BinOp::And | BinOp::Or => {
                    if l != Type::Bool || r != Type::Bool {
                        bail!("logical op expects bool");
                    }
                    Ok(Type::Bool)
                }
            }
        }
        Expr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            let c = check_expr(cond, locals, fun_types)?;
            if c != Type::Bool {
                bail!("if condition must be bool");
            }
            let t = check_expr(then_expr, locals, fun_types)?;
            let e = check_expr(else_expr, locals, fun_types)?;
            if t != e {
                bail!("if branches must have same type");
            }
            Ok(t)
        }
        Expr::While { cond, body } => {
            let c = check_expr(cond, locals, fun_types)?;
            if c != Type::Bool {
                bail!("while condition must be bool");
            }
            let _ = check_expr(body, locals, fun_types)?;
            Ok(Type::Unit)
        }
        Expr::Call { name, args } => {
            let (params, ret) = fun_types
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("unknown function {}", name))?;
            if params.len() != args.len() {
                bail!("call {} arg count mismatch", name);
            }
            for (arg, expected) in args.iter().zip(params.iter()) {
                let a = check_expr(arg, locals, fun_types)?;
                if &a != expected {
                    bail!("call {} arg type mismatch", name);
                }
            }
            Ok(ret.clone())
        }
        Expr::Block { stmts, expr } => {
            let mut scoped = locals.clone();
            let mut defined = std::collections::HashSet::new();
            for stmt in stmts {
                match stmt {
                    Stmt::Let { name, ty, value } => {
                        if defined.contains(name) {
                            bail!("duplicate let in same block: {}", name);
                        }
                        let vty = check_expr(value, &scoped, fun_types)?;
                        if &vty != ty {
                            bail!("let {} type mismatch", name);
                        }
                        scoped.insert(name.clone(), ty.clone());
                        defined.insert(name.clone());
                    }
                    Stmt::Expr(expr) => {
                        let t = check_expr(expr, &scoped, fun_types)?;
                        if t == Type::Unit {
                            continue;
                        }
                    }
                }
            }
            check_expr(expr, &scoped, fun_types)
        }
    }
}
