use crate::ast::{BinOp, Expr, Function, Program, Stmt, Type, UnaryOp};
use crate::ir::{BasicBlock, Instr, Module, Param, Terminator};
use anyhow::Result;
use std::collections::HashMap;

pub fn lower_to_ir(program: Program) -> Result<Module> {
    let mut fun_types = HashMap::new();
    for f in &program.functions {
        fun_types.insert(
            f.name.clone(),
            (
                f.params.iter().map(|p| p.ty.clone()).collect::<Vec<_>>(),
                f.return_type.clone(),
            ),
        );
    }

    let mut functions = Vec::new();
    for f in &program.functions {
        functions.push(lower_function(f, &fun_types)?);
    }
    Ok(Module { functions })
}

fn lower_function(
    func: &Function,
    fun_types: &HashMap<String, (Vec<Type>, Type)>,
) -> Result<crate::ir::Function> {
    let mut builder = FunctionBuilder::new(fun_types);
    let entry = builder.new_block("entry");
    builder.set_current(entry);

    for p in &func.params {
        builder.locals_ty.insert(p.name.clone(), p.ty.clone());
    }

    let value = builder.emit_expr(&func.body)?;
    builder.set_terminator(Terminator::Ret { value });

    Ok(crate::ir::Function {
        name: func.name.clone(),
        params: func
            .params
            .iter()
            .map(|p| Param {
                name: p.name.clone(),
                ty: p.ty.clone(),
            })
            .collect(),
        return_type: func.return_type.clone(),
        blocks: builder.blocks,
    })
}

struct FunctionBuilder {
    blocks: Vec<BasicBlock>,
    current: usize,
    temp: usize,
    label: usize,
    locals: HashMap<String, String>,
    locals_ty: HashMap<String, Type>,
    fun_types: HashMap<String, (Vec<Type>, Type)>,
}

impl FunctionBuilder {
    fn new(fun_types: &HashMap<String, (Vec<Type>, Type)>) -> Self {
        Self {
            blocks: Vec::new(),
            current: 0,
            temp: 0,
            label: 0,
            locals: HashMap::new(),
            locals_ty: HashMap::new(),
            fun_types: fun_types.clone(),
        }
    }

    fn new_block(&mut self, name: &str) -> usize {
        let idx = self.blocks.len();
        self.blocks.push(BasicBlock {
            name: name.to_string(),
            instrs: Vec::new(),
            term: Terminator::Ret {
                value: "0".to_string(),
            },
        });
        idx
    }

    fn set_current(&mut self, idx: usize) {
        self.current = idx;
    }

    fn current_block_mut(&mut self) -> &mut BasicBlock {
        &mut self.blocks[self.current]
    }

    fn set_terminator(&mut self, term: Terminator) {
        self.blocks[self.current].term = term;
    }

    fn new_temp(&mut self) -> String {
        self.temp += 1;
        format!("%t{}", self.temp)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label += 1;
        format!("{}{}", prefix, self.label)
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Int(v) => Ok(v.to_string()),
            Expr::Bool(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
            Expr::Ident(name) => {
                if let Some(ptr) = self.locals.get(name).cloned() {
                    let ty = self.type_of_ident(name)?;
                    let dest = self.new_temp();
                    self.current_block_mut()
                        .instrs
                        .push(Instr::Load {
                            dest: dest.clone(),
                            ty,
                            ptr,
                        });
                    Ok(dest)
                } else {
                    Ok(format!("%{}", name))
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                let l = self.emit_expr(lhs)?;
                let r = self.emit_expr(rhs)?;
                let dest = self.new_temp();
                match op {
                    BinOp::Add => self.current_block_mut().instrs.push(Instr::Add {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Sub => self.current_block_mut().instrs.push(Instr::Sub {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Mul => self.current_block_mut().instrs.push(Instr::Mul {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Div => self.current_block_mut().instrs.push(Instr::SDiv {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Rem => self.current_block_mut().instrs.push(Instr::SRem {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Lt => self.current_block_mut().instrs.push(Instr::ICmp {
                        dest: dest.clone(),
                        pred: "slt".to_string(),
                        ty: Type::I32,
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Le => self.current_block_mut().instrs.push(Instr::ICmp {
                        dest: dest.clone(),
                        pred: "sle".to_string(),
                        ty: Type::I32,
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Gt => self.current_block_mut().instrs.push(Instr::ICmp {
                        dest: dest.clone(),
                        pred: "sgt".to_string(),
                        ty: Type::I32,
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Ge => self.current_block_mut().instrs.push(Instr::ICmp {
                        dest: dest.clone(),
                        pred: "sge".to_string(),
                        ty: Type::I32,
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Eq => {
                        let ty = self.type_of_expr(lhs)?;
                        self.current_block_mut().instrs.push(Instr::ICmp {
                            dest: dest.clone(),
                            pred: "eq".to_string(),
                            ty,
                            lhs: l,
                            rhs: r,
                        })
                    }
                    BinOp::Ne => {
                        let ty = self.type_of_expr(lhs)?;
                        self.current_block_mut().instrs.push(Instr::ICmp {
                            dest: dest.clone(),
                            pred: "ne".to_string(),
                            ty,
                            lhs: l,
                            rhs: r,
                        })
                    }
                    BinOp::And => self.current_block_mut().instrs.push(Instr::And {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                    BinOp::Or => self.current_block_mut().instrs.push(Instr::Or {
                        dest: dest.clone(),
                        lhs: l,
                        rhs: r,
                    }),
                }
                Ok(dest)
            }
            Expr::Unary { op, expr } => {
                let v = self.emit_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        let dest = self.new_temp();
                        self.current_block_mut().instrs.push(Instr::Sub {
                            dest: dest.clone(),
                            lhs: "0".to_string(),
                            rhs: v,
                        });
                        Ok(dest)
                    }
                    UnaryOp::Not => {
                        let dest = self.new_temp();
                        self.current_block_mut().instrs.push(Instr::Xor {
                            dest: dest.clone(),
                            lhs: v,
                            rhs: "true".to_string(),
                        });
                        Ok(dest)
                    }
                }
            }
            Expr::Call { name, args } => {
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.emit_expr(a)?);
                }
                let dest = self.new_temp();
                let ret_ty = self
                    .fun_types
                    .get(name)
                    .map(|(_, ret)| ret.clone())
                    .unwrap_or(Type::I32);
                let param_tys = self
                    .fun_types
                    .get(name)
                    .map(|(p, _)| p.clone())
                    .unwrap_or_default();
                let args_typed = arg_vals
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let ty = param_tys.get(i).cloned().unwrap_or(Type::I32);
                        (ty, v)
                    })
                    .collect::<Vec<_>>();
                self.current_block_mut().instrs.push(Instr::Call {
                    dest: Some(dest.clone()),
                    ret_ty,
                    func: name.clone(),
                    args: args_typed,
                });
                Ok(dest)
            }
            Expr::If {
                cond,
                then_expr,
                else_expr,
            } => {
                let cond_val = self.emit_expr(cond)?;
                let then_name = self.new_label("then");
                let else_name = self.new_label("else");
                let merge_name = self.new_label("merge");

                let then_idx = self.new_block(&then_name);
                let else_idx = self.new_block(&else_name);
                let merge_idx = self.new_block(&merge_name);

                self.set_terminator(Terminator::CondBr {
                    cond: cond_val,
                    then_dest: then_name.clone(),
                    else_dest: else_name.clone(),
                });

                self.set_current(then_idx);
                let then_val = self.emit_expr(then_expr)?;
                self.set_terminator(Terminator::Br {
                    dest: merge_name.clone(),
                });

                self.set_current(else_idx);
                let else_val = self.emit_expr(else_expr)?;
                self.set_terminator(Terminator::Br {
                    dest: merge_name.clone(),
                });

                self.set_current(merge_idx);
                let dest = self.new_temp();
                let phi_ty = self.type_of_expr(then_expr)?;
                self.current_block_mut().instrs.push(Instr::Phi {
                    dest: dest.clone(),
                    ty: phi_ty,
                    incomings: vec![
                        (then_val, then_name),
                        (else_val, else_name),
                    ],
                });
                Ok(dest)
            }
            Expr::While { cond, body } => {
                let cond_name = self.new_label("cond");
                let body_name = self.new_label("body");
                let exit_name = self.new_label("exit");

                let cond_idx = self.new_block(&cond_name);
                let body_idx = self.new_block(&body_name);
                let exit_idx = self.new_block(&exit_name);

                self.set_terminator(Terminator::Br {
                    dest: cond_name.clone(),
                });

                self.set_current(cond_idx);
                let cond_val = self.emit_expr(cond)?;
                self.set_terminator(Terminator::CondBr {
                    cond: cond_val,
                    then_dest: body_name.clone(),
                    else_dest: exit_name.clone(),
                });

                self.set_current(body_idx);
                let _ = self.emit_expr(body)?;
                self.set_terminator(Terminator::Br {
                    dest: cond_name.clone(),
                });

                self.set_current(exit_idx);
                Ok("0".to_string())
            }
            Expr::Block { stmts, expr } => {
                for stmt in stmts {
                    self.emit_stmt(stmt)?;
                }
                self.emit_expr(expr)
            }
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let ptr = format!("%{}.addr", name);
                if !self.locals.contains_key(name) {
                    let ty = self.type_of_expr(value)?;
                    self.blocks[0]
                        .instrs
                        .insert(0, Instr::Alloca { dest: ptr.clone(), ty });
                    self.locals.insert(name.clone(), ptr.clone());
                }
                let val_ty = self.type_of_expr(value)?;
                let val = self.emit_expr(value)?;
                self.current_block_mut()
                    .instrs
                    .push(Instr::Store {
                        ty: val_ty.clone(),
                        value: val,
                        ptr,
                    });
                self.locals_ty.entry(name.clone()).or_insert(val_ty);
                Ok(())
            }
            Stmt::Expr(expr) => {
                let _ = self.emit_expr(expr)?;
                Ok(())
            }
        }
    }

    fn type_of_ident(&self, name: &str) -> Result<Type> {
        self.locals_ty
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown identifier {}", name))
    }

    fn type_of_expr(&self, expr: &Expr) -> Result<Type> {
        self.type_of_expr_scoped(expr, &self.locals_ty)
    }

    fn type_of_expr_scoped(
        &self,
        expr: &Expr,
        locals: &HashMap<String, Type>,
    ) -> Result<Type> {
        match expr {
            Expr::Int(_) => Ok(Type::I32),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Ident(name) => locals
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown identifier {}", name)),
            Expr::Unary { op, expr } => {
                let t = self.type_of_expr_scoped(expr, locals)?;
                match op {
                    UnaryOp::Neg => Ok(t),
                    UnaryOp::Not => Ok(Type::Bool),
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                let _ = self.type_of_expr_scoped(lhs, locals)?;
                let _ = self.type_of_expr_scoped(rhs, locals)?;
                match op {
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Rem => Ok(Type::I32),
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Ok(Type::Bool),
                    BinOp::Eq | BinOp::Ne => Ok(Type::Bool),
                    BinOp::And | BinOp::Or => Ok(Type::Bool),
                }
            }
            Expr::If { then_expr, .. } => self.type_of_expr_scoped(then_expr, locals),
            Expr::Call { name, .. } => self
                .fun_types
                .get(name)
                .map(|(_, ret)| ret.clone())
                .ok_or_else(|| anyhow::anyhow!("unknown function {}", name)),
            Expr::Block { stmts, expr } => {
                let mut scoped = locals.clone();
                for stmt in stmts {
                    match stmt {
                        Stmt::Let { name, value, .. } => {
                            let t = self.type_of_expr_scoped(value, &scoped)?;
                            scoped.insert(name.clone(), t);
                        }
                        Stmt::Expr(e) => {
                            let _ = self.type_of_expr_scoped(e, &scoped)?;
                        }
                    }
                }
                self.type_of_expr_scoped(expr, &scoped)
            }
            Expr::While { .. } => Ok(Type::Unit),
        }
    }
}
