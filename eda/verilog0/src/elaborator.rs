/// Elaborator - semantic analysis, symbol resolution, and design flattening
/// Produces an elaborated design ready for code generation.

use crate::parser::*;
use std::collections::HashMap;
use std::fmt;

// ============================================================
// Elaborated Design
// ============================================================

#[derive(Debug)]
pub struct Design {
    pub top_module: String,
    pub modules: HashMap<String, ElabModule>,
}

#[derive(Debug, Clone)]
pub struct ElabModule {
    pub name: String,
    pub ports: Vec<ElabPort>,
    pub signals: HashMap<String, Signal>,
    pub continuous_assigns: Vec<ElabContAssign>,
    pub always_blocks: Vec<ElabAlways>,
    pub initial_blocks: Vec<ElabInitial>,
    pub instances: Vec<ElabInstance>,
    pub gate_instances: Vec<ElabGate>,
}

#[derive(Debug, Clone)]
pub struct ElabPort {
    pub name: String,
    pub dir: PortDir,
    pub net_type: NetType,
    pub width: u32,
    pub msb: i32,
    pub lsb: i32,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: String,
    pub kind: SignalKind,
    pub width: u32,
    pub msb: i32,
    pub lsb: i32,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalKind {
    Wire,
    Reg,
    Integer,
    Real,
    Parameter(i64),
}

#[derive(Debug, Clone)]
pub struct ElabContAssign {
    pub lhs: ElabExpr,
    pub rhs: ElabExpr,
}

#[derive(Debug, Clone)]
pub struct ElabAlways {
    pub sensitivity: Sensitivity,
    pub body: ElabStmt,
}

#[derive(Debug, Clone)]
pub struct ElabInitial {
    pub body: ElabStmt,
}

#[derive(Debug, Clone)]
pub struct ElabInstance {
    pub module_name: String,
    pub inst_name: String,
    pub port_connections: Vec<(String, Option<ElabExpr>)>,
}

#[derive(Debug, Clone)]
pub struct ElabGate {
    pub gate_type: GateType,
    pub inst_name: String,
    pub output: ElabExpr,
    pub inputs: Vec<ElabExpr>,
}

#[derive(Debug, Clone)]
pub enum ElabStmt {
    Block(Vec<ElabStmt>),
    ForkJoin(Vec<ElabStmt>),
    If(Box<ElabExpr>, Box<ElabStmt>, Option<Box<ElabStmt>>),
    Case(ElabExpr, CaseKind, Vec<(Vec<ElabExpr>, ElabStmt)>, Option<Box<ElabStmt>>),
    For(Box<ElabStmt>, ElabExpr, Box<ElabStmt>, Box<ElabStmt>),
    While(ElabExpr, Box<ElabStmt>),
    Repeat(ElabExpr, Box<ElabStmt>),
    Forever(Box<ElabStmt>),
    BlockingAssign(ElabExpr, ElabExpr),
    NonBlockingAssign(ElabExpr, ElabExpr),
    EventControl(Sensitivity, Box<ElabStmt>),
    DelayControl(ElabExpr, Box<ElabStmt>),
    SystemTask(String, Vec<ElabExpr>),
    TaskCall(String, Vec<ElabExpr>),
    Disable(String),
    Null,
}

#[derive(Debug, Clone)]
pub enum ElabExpr {
    IntLit(u64, u32),   // value, width
    RealLit(f64),
    StringLit(String),
    Signal(String, u32),   // name, width
    Index(Box<ElabExpr>, Box<ElabExpr>),
    Slice(Box<ElabExpr>, Box<ElabExpr>, Box<ElabExpr>),
    Concat(Vec<ElabExpr>, u32),
    Repeat(Box<ElabExpr>, Vec<ElabExpr>, u32),
    Unary(UnaryOp, Box<ElabExpr>, u32),
    Binary(BinaryOp, Box<ElabExpr>, Box<ElabExpr>, u32),
    Ternary(Box<ElabExpr>, Box<ElabExpr>, Box<ElabExpr>, u32),
    FuncCall(String, Vec<ElabExpr>),
    SystemCall(String, Vec<ElabExpr>),
    HiZ,
}

// ============================================================
// Elaboration Error
// ============================================================

#[derive(Debug)]
pub struct ElabError {
    pub msg: String,
}

impl fmt::Display for ElabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Elaboration error: {}", self.msg)
    }
}

// ============================================================
// Elaborator
// ============================================================

pub fn elaborate(src: SourceFile, top: Option<&str>) -> Result<Design, ElabError> {
    let mut elab = Elaborator::new();
    elab.run(src, top)
}

struct Elaborator {
    module_defs: HashMap<String, Module>,
}

impl Elaborator {
    fn new() -> Self {
        Elaborator {
            module_defs: HashMap::new(),
        }
    }

    fn run(&mut self, src: SourceFile, top: Option<&str>) -> Result<Design, ElabError> {
        // Register all modules
        for module in src.modules {
            self.module_defs.insert(module.name.clone(), module);
        }

        // Find top module
        let top_name = if let Some(t) = top {
            t.to_string()
        } else {
            // Use last module as top (common convention)
            self.module_defs.keys().last()
                .ok_or_else(|| ElabError { msg: "No modules found".to_string() })?
                .clone()
        };

        if !self.module_defs.contains_key(&top_name) {
            return Err(ElabError {
                msg: format!("Top module '{}' not found", top_name),
            });
        }

        // Elaborate all modules
        let mut modules = HashMap::new();
        let all_names: Vec<String> = self.module_defs.keys().cloned().collect();
        for name in all_names {
            let elab_mod = self.elaborate_module(&name)?;
            modules.insert(name, elab_mod);
        }

        Ok(Design { top_module: top_name, modules })
    }

    fn elaborate_module(&self, name: &str) -> Result<ElabModule, ElabError> {
        let module = self.module_defs[name].clone();
        let mut signals: HashMap<String, Signal> = HashMap::new();

        // Process port declarations from module header
        let mut ports = Vec::new();
        for port_decl in &module.ports {
            let (msb, lsb, width) = self.resolve_range(&port_decl.range, &signals);
            for port_name in &port_decl.names {
                let net_type = port_decl.net_type.clone().unwrap_or(
                    if port_decl.dir == PortDir::Output { NetType::Wire } else { NetType::Wire }
                );
                ports.push(ElabPort {
                    name: port_name.clone(),
                    dir: port_decl.dir.clone(),
                    net_type: net_type.clone(),
                    width,
                    msb,
                    lsb,
                });
                let kind = match net_type {
                    NetType::Reg     => SignalKind::Reg,
                    NetType::Integer => SignalKind::Integer,
                    NetType::Real    => SignalKind::Real,
                    _                => SignalKind::Wire,
                };
                signals.insert(port_name.clone(), Signal {
                    name: port_name.clone(),
                    kind,
                    width,
                    msb,
                    lsb,
                    signed: port_decl.signed,
                });
            }
        }

        // First pass: collect all declarations
        for item in &module.items {
            match item {
                ModuleItem::PortDecl(pd) => {
                    let (msb, lsb, width) = self.resolve_range(&pd.range, &signals);
                    for pname in &pd.names {
                        let kind = match &pd.net_type {
                            Some(NetType::Reg)     => SignalKind::Reg,
                            Some(NetType::Integer) => SignalKind::Integer,
                            Some(NetType::Real)    => SignalKind::Real,
                            _                      => SignalKind::Wire,
                        };
                        // Update or insert port
                        if let Some(p) = ports.iter_mut().find(|p| &p.name == pname) {
                            p.dir = pd.dir.clone();
                            p.width = width;
                            p.msb = msb;
                            p.lsb = lsb;
                        } else {
                            ports.push(ElabPort {
                                name: pname.clone(),
                                dir: pd.dir.clone(),
                                net_type: pd.net_type.clone().unwrap_or(NetType::Wire),
                                width, msb, lsb,
                            });
                        }
                        signals.entry(pname.clone()).or_insert(Signal {
                            name: pname.clone(), kind,
                            width, msb, lsb, signed: pd.signed,
                        });
                    }
                }
                ModuleItem::NetDecl(nd) => {
                    let (msb, lsb, width) = self.resolve_range(&nd.range, &signals);
                    let kind = match nd.net_type {
                        NetType::Reg     => SignalKind::Reg,
                        NetType::Integer => SignalKind::Integer,
                        NetType::Real    => SignalKind::Real,
                        _                => SignalKind::Wire,
                    };
                    for (sname, _) in &nd.names {
                        signals.entry(sname.clone()).or_insert(Signal {
                            name: sname.clone(), kind: kind.clone(),
                            width, msb, lsb, signed: nd.signed,
                        });
                    }
                }
                ModuleItem::ParamDecl(pd) => {
                    for (pname, expr) in &pd.params {
                        let val = self.const_eval(expr, &signals).unwrap_or(0);
                        signals.insert(pname.clone(), Signal {
                            name: pname.clone(),
                            kind: SignalKind::Parameter(val),
                            width: 32, msb: 31, lsb: 0, signed: false,
                        });
                    }
                }
                _ => {}
            }
        }

        // Second pass: elaborate behavioral items
        let mut continuous_assigns = Vec::new();
        let mut always_blocks = Vec::new();
        let mut initial_blocks = Vec::new();
        let mut instances = Vec::new();
        let mut gate_instances = Vec::new();

        for item in &module.items {
            match item {
                ModuleItem::ContinuousAssign(ca) => {
                    let lhs = self.elab_expr(&ca.lhs, &signals)?;
                    let rhs = self.elab_expr(&ca.rhs, &signals)?;
                    continuous_assigns.push(ElabContAssign { lhs, rhs });
                }
                ModuleItem::AlwaysBlock(ab) => {
                    let body = self.elab_stmt(&ab.body, &signals)?;
                    always_blocks.push(ElabAlways {
                        sensitivity: ab.sensitivity.clone(),
                        body,
                    });
                }
                ModuleItem::InitialBlock(ib) => {
                    let body = self.elab_stmt(&ib.body, &signals)?;
                    initial_blocks.push(ElabInitial { body });
                }
                ModuleItem::ModuleInst(mi) => {
                    for inst in &mi.instances {
                        let mut port_connections = Vec::new();
                        for conn in &inst.connections {
                            match conn {
                                PortConnect::ByName(pname, expr) => {
                                    let elab_expr = if let Some(e) = expr {
                                        Some(self.elab_expr(e, &signals)?)
                                    } else {
                                        None
                                    };
                                    port_connections.push((pname.clone(), elab_expr));
                                }
                                PortConnect::ByPosition(expr) => {
                                    let elab_expr = self.elab_expr(expr, &signals)?;
                                    port_connections.push((format!("_{}", port_connections.len()), Some(elab_expr)));
                                }
                            }
                        }
                        instances.push(ElabInstance {
                            module_name: mi.module_name.clone(),
                            inst_name: inst.name.clone(),
                            port_connections,
                        });
                    }
                }
                ModuleItem::GateInst(gi) => {
                    for inst in &gi.instances {
                        if inst.terminals.len() >= 2 {
                            let output = self.elab_expr(&inst.terminals[0], &signals)?;
                            let inputs = inst.terminals[1..].iter()
                                .map(|e| self.elab_expr(e, &signals))
                                .collect::<Result<Vec<_>, _>>()?;
                            gate_instances.push(ElabGate {
                                gate_type: gi.gate_type.clone(),
                                inst_name: inst.name.clone().unwrap_or_else(|| {
                                    format!("_gate_{}", gate_instances.len())
                                }),
                                output,
                                inputs,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(ElabModule {
            name: name.to_string(),
            ports,
            signals,
            continuous_assigns,
            always_blocks,
            initial_blocks,
            instances,
            gate_instances,
        })
    }

    fn resolve_range(&self, range: &Option<Range>, signals: &HashMap<String, Signal>) -> (i32, i32, u32) {
        if let Some(r) = range {
            let msb = self.const_eval(&r.msb, signals).unwrap_or(0) as i32;
            let lsb = self.const_eval(&r.lsb, signals).unwrap_or(0) as i32;
            let width = (msb - lsb).abs() as u32 + 1;
            (msb, lsb, width)
        } else {
            (0, 0, 1)
        }
    }

    fn const_eval(&self, expr: &Expr, signals: &HashMap<String, Signal>) -> Option<i64> {
        match expr {
            Expr::IntLit(n) => Some(*n as i64),
            Expr::BasedLit { size: _, base, value } => {
                let clean = value.replace('_', "");
                let radix = match base {
                    'b' | 'B' => 2,
                    'o' | 'O' => 8,
                    'd' | 'D' => 10,
                    'h' | 'H' => 16,
                    _ => 10,
                };
                i64::from_str_radix(&clean, radix).ok()
            }
            Expr::Ident(name) => {
                if let Some(sig) = signals.get(name) {
                    if let SignalKind::Parameter(v) = sig.kind {
                        return Some(v);
                    }
                }
                None
            }
            Expr::Binary(op, lhs, rhs) => {
                let l = self.const_eval(lhs, signals)?;
                let r = self.const_eval(rhs, signals)?;
                Some(match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => if r != 0 { l / r } else { 0 },
                    BinaryOp::Mod => if r != 0 { l % r } else { 0 },
                    _ => return None,
                })
            }
            Expr::Unary(op, e) => {
                let v = self.const_eval(e, signals)?;
                Some(match op {
                    UnaryOp::Minus => -v,
                    UnaryOp::Plus  => v,
                    UnaryOp::BitNot => !v,
                    _ => return None,
                })
            }
            _ => None,
        }
    }

    fn elab_expr(&self, expr: &Expr, signals: &HashMap<String, Signal>) -> Result<ElabExpr, ElabError> {
        match expr {
            Expr::IntLit(n) => Ok(ElabExpr::IntLit(*n, 32)),
            Expr::BasedLit { size, base, value } => {
                let w = size.unwrap_or(32);
                // Convert to integer value
                let clean = value.replace('_', "");
                // Replace x/z with 0 for now (simplified)
                let clean = clean.replace('x', "0").replace('X', "0")
                                 .replace('z', "0").replace('Z', "0");
                let radix = match base {
                    'b' | 'B' => 2u32,
                    'o' | 'O' => 8,
                    'h' | 'H' => 16,
                    _ => 10,
                };
                let val = u64::from_str_radix(&clean, radix).unwrap_or(0);
                Ok(ElabExpr::IntLit(val, w))
            }
            Expr::RealLit(f) => Ok(ElabExpr::RealLit(*f)),
            Expr::StringLit(s) => Ok(ElabExpr::StringLit(s.clone())),
            Expr::Ident(name) => {
                let width = signals.get(name).map_or(1, |s| s.width);
                Ok(ElabExpr::Signal(name.clone(), width))
            }
            Expr::Index(base, idx) => {
                let b = self.elab_expr(base, signals)?;
                let i = self.elab_expr(idx, signals)?;
                Ok(ElabExpr::Index(Box::new(b), Box::new(i)))
            }
            Expr::Slice(base, msb, lsb) => {
                let b = self.elab_expr(base, signals)?;
                let m = self.elab_expr(msb, signals)?;
                let l = self.elab_expr(lsb, signals)?;
                Ok(ElabExpr::Slice(Box::new(b), Box::new(m), Box::new(l)))
            }
            Expr::Concat(parts) => {
                let elabs: Result<Vec<_>, _> = parts.iter()
                    .map(|e| self.elab_expr(e, signals))
                    .collect();
                let elabs = elabs?;
                let width: u32 = elabs.iter().map(|e| expr_width(e)).sum();
                Ok(ElabExpr::Concat(elabs, width))
            }
            Expr::Repeat(count, parts) => {
                let ec = self.elab_expr(count, signals)?;
                let ep: Result<Vec<_>, _> = parts.iter()
                    .map(|e| self.elab_expr(e, signals))
                    .collect();
                let ep = ep?;
                let inner_w: u32 = ep.iter().map(|e| expr_width(e)).sum();
                let rep = self.const_eval(count, signals).unwrap_or(1) as u32;
                Ok(ElabExpr::Repeat(Box::new(ec), ep, inner_w * rep))
            }
            Expr::Unary(op, e) => {
                let ee = self.elab_expr(e, signals)?;
                let w = match op {
                    UnaryOp::Not | UnaryOp::RedAnd | UnaryOp::RedNand
                    | UnaryOp::RedOr | UnaryOp::RedNor | UnaryOp::RedXor
                    | UnaryOp::RedXnor => 1,
                    _ => expr_width(&ee),
                };
                Ok(ElabExpr::Unary(op.clone(), Box::new(ee), w))
            }
            Expr::Binary(op, lhs, rhs) => {
                let el = self.elab_expr(lhs, signals)?;
                let er = self.elab_expr(rhs, signals)?;
                let w = match op {
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::CaseEq | BinaryOp::CaseNe
                    | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
                    | BinaryOp::LogAnd | BinaryOp::LogOr => 1,
                    _ => expr_width(&el).max(expr_width(&er)),
                };
                Ok(ElabExpr::Binary(op.clone(), Box::new(el), Box::new(er), w))
            }
            Expr::Ternary(cond, then_, else_) => {
                let ec = self.elab_expr(cond, signals)?;
                let et = self.elab_expr(then_, signals)?;
                let ee = self.elab_expr(else_, signals)?;
                let w = expr_width(&et).max(expr_width(&ee));
                Ok(ElabExpr::Ternary(Box::new(ec), Box::new(et), Box::new(ee), w))
            }
            Expr::FuncCall(name, args) => {
                let ea: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.elab_expr(a, signals))
                    .collect();
                Ok(ElabExpr::FuncCall(name.clone(), ea?))
            }
            Expr::SystemCall(name, args) => {
                let ea: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.elab_expr(a, signals))
                    .collect();
                Ok(ElabExpr::SystemCall(name.clone(), ea?))
            }
            // 找到此區塊末尾，添加 Member 分支：
            Expr::X | Expr::Z | Expr::HiZ => Ok(ElabExpr::HiZ),
            Expr::Member(_, _) => Err(ElabError { msg: "Struct member access is not supported yet".into() }),
        }
    }

    fn elab_stmt(&self, stmt: &Stmt, signals: &HashMap<String, Signal>) -> Result<ElabStmt, ElabError> {
        match stmt {
            Stmt::Block(stmts) => {
                let es: Result<Vec<_>, _> = stmts.iter()
                    .map(|s| self.elab_stmt(s, signals))
                    .collect();
                Ok(ElabStmt::Block(es?))
            }
            Stmt::ForkJoin(stmts) => {
                let es: Result<Vec<_>, _> = stmts.iter()
                    .map(|s| self.elab_stmt(s, signals))
                    .collect();
                Ok(ElabStmt::ForkJoin(es?))
            }
            Stmt::If(is) => {
                let cond = self.elab_expr(&is.cond, signals)?;
                let then_ = self.elab_stmt(&is.then_, signals)?;
                let else_ = if let Some(e) = &is.else_ {
                    Some(Box::new(self.elab_stmt(e, signals)?))
                } else {
                    None
                };
                Ok(ElabStmt::If(Box::new(cond), Box::new(then_), else_))
            }
            Stmt::Case(cs) => {
                let expr = self.elab_expr(&cs.expr, signals)?;
                let items: Result<Vec<_>, _> = cs.items.iter().map(|item| {
                    let patterns: Result<Vec<_>, _> = item.patterns.iter()
                        .map(|p| self.elab_expr(p, signals))
                        .collect();
                    let stmt = self.elab_stmt(&item.stmt, signals)?;
                    Ok((patterns?, stmt))
                }).collect();
                let default = if let Some(d) = &cs.default {
                    Some(Box::new(self.elab_stmt(d, signals)?))
                } else {
                    None
                };
                Ok(ElabStmt::Case(expr, cs.kind.clone(), items?, default))
            }
            Stmt::For(fs) => {
                let init = self.elab_stmt(&fs.init, signals)?;
                let cond = self.elab_expr(&fs.cond, signals)?;
                let step = self.elab_stmt(&fs.step, signals)?;
                let body = self.elab_stmt(&fs.body, signals)?;
                Ok(ElabStmt::For(Box::new(init), cond, Box::new(step), Box::new(body)))
            }
            Stmt::While(ws) => {
                let cond = self.elab_expr(&ws.cond, signals)?;
                let body = self.elab_stmt(&ws.body, signals)?;
                Ok(ElabStmt::While(cond, Box::new(body)))
            }
            Stmt::Repeat(rs) => {
                let count = self.elab_expr(&rs.count, signals)?;
                let body = self.elab_stmt(&rs.body, signals)?;
                Ok(ElabStmt::Repeat(count, Box::new(body)))
            }
            Stmt::Forever(body) => {
                let b = self.elab_stmt(body, signals)?;
                Ok(ElabStmt::Forever(Box::new(b)))
            }
            Stmt::BlockingAssign(lhs, rhs) => {
                Ok(ElabStmt::BlockingAssign(
                    self.elab_expr(lhs, signals)?,
                    self.elab_expr(rhs, signals)?,
                ))
            }
            Stmt::NonBlockingAssign(lhs, rhs) => {
                Ok(ElabStmt::NonBlockingAssign(
                    self.elab_expr(lhs, signals)?,
                    self.elab_expr(rhs, signals)?,
                ))
            }
            Stmt::EventControl(sens, body) => {
                let b = self.elab_stmt(body, signals)?;
                Ok(ElabStmt::EventControl(sens.clone(), Box::new(b)))
            }
            Stmt::DelayControl(delay, body) => {
                let d = self.elab_expr(delay, signals)?;
                let b = self.elab_stmt(body, signals)?;
                Ok(ElabStmt::DelayControl(d, Box::new(b)))
            }
            Stmt::SystemTask(name, args) => {
                let ea: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.elab_expr(a, signals))
                    .collect();
                Ok(ElabStmt::SystemTask(name.clone(), ea?))
            }
            Stmt::TaskCall(name, args) => {
                let ea: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.elab_expr(a, signals))
                    .collect();
                Ok(ElabStmt::TaskCall(name.clone(), ea?))
            }
            Stmt::Disable(name) => Ok(ElabStmt::Disable(name.clone())),
            Stmt::Null => Ok(ElabStmt::Null),
        }
    }
}

pub fn expr_width(e: &ElabExpr) -> u32 {
    match e {
        ElabExpr::IntLit(_, w)  => *w,
        ElabExpr::RealLit(_)    => 64,
        ElabExpr::StringLit(s)  => (s.len() as u32) * 8,
        ElabExpr::Signal(_, w)  => *w,
        ElabExpr::Index(_, _)   => 1,
        ElabExpr::Slice(_, m, l) => {
            let mw = match m.as_ref() { ElabExpr::IntLit(v, _) => *v as i32, _ => 0 };
            let lw = match l.as_ref() { ElabExpr::IntLit(v, _) => *v as i32, _ => 0 };
            (mw - lw).unsigned_abs() + 1
        }
        ElabExpr::Concat(_, w)    => *w,
        ElabExpr::Repeat(_, _, w) => *w,
        ElabExpr::Unary(_, _, w)  => *w,
        ElabExpr::Binary(_, _, _, w) => *w,
        ElabExpr::Ternary(_, _, _, w) => *w,
        ElabExpr::FuncCall(_, _)    => 32,
        ElabExpr::SystemCall(_, _)  => 32,
        ElabExpr::HiZ               => 1,
    }
}
