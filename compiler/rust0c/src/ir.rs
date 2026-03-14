use crate::ast::Type;

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: String,
    pub instrs: Vec<Instr>,
    pub term: Terminator,
}

#[derive(Debug, Clone)]
pub enum Instr {
    Add { dest: String, lhs: String, rhs: String },
    Sub { dest: String, lhs: String, rhs: String },
    Mul { dest: String, lhs: String, rhs: String },
    SDiv { dest: String, lhs: String, rhs: String },
    SRem { dest: String, lhs: String, rhs: String },
    And { dest: String, lhs: String, rhs: String },
    Or { dest: String, lhs: String, rhs: String },
    Xor { dest: String, lhs: String, rhs: String },
    ICmp { dest: String, pred: String, ty: Type, lhs: String, rhs: String },
    Call { dest: Option<String>, ret_ty: Type, func: String, args: Vec<(Type, String)> },
    Alloca { dest: String, ty: Type },
    Store { ty: Type, value: String, ptr: String },
    Load { dest: String, ty: Type, ptr: String },
    Phi { dest: String, ty: Type, incomings: Vec<(String, String)> },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret { value: String },
    Br { dest: String },
    CondBr { cond: String, then_dest: String, else_dest: String },
}

impl ToString for Module {
    fn to_string(&self) -> String {
        let mut out = String::new();
        for func in &self.functions {
            let params = func
                .params
                .iter()
                .map(|p| format!("{} %{}", p.ty.to_ll(), p.name))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "define {} @{}({}) {{\n",
                func.return_type.to_ll(),
                func.name,
                params
            ));
            for bb in &func.blocks {
                out.push_str(&format!("{}:\n", bb.name));
                for instr in &bb.instrs {
                    out.push_str(&format!("  {}\n", instr.to_string()));
                }
                out.push_str(&format!("  {}\n", bb.term.to_string()));
                out.push('\n');
            }
            out.push_str("}\n\n");
        }
        out
    }
}

impl ToString for Instr {
    fn to_string(&self) -> String {
        match self {
            Instr::Add { dest, lhs, rhs } => format!("{} = add i32 {}, {}", dest, lhs, rhs),
            Instr::Sub { dest, lhs, rhs } => format!("{} = sub i32 {}, {}", dest, lhs, rhs),
            Instr::Mul { dest, lhs, rhs } => format!("{} = mul i32 {}, {}", dest, lhs, rhs),
            Instr::SDiv { dest, lhs, rhs } => format!("{} = sdiv i32 {}, {}", dest, lhs, rhs),
            Instr::SRem { dest, lhs, rhs } => format!("{} = srem i32 {}, {}", dest, lhs, rhs),
            Instr::And { dest, lhs, rhs } => format!("{} = and i1 {}, {}", dest, lhs, rhs),
            Instr::Or { dest, lhs, rhs } => format!("{} = or i1 {}, {}", dest, lhs, rhs),
            Instr::Xor { dest, lhs, rhs } => format!("{} = xor i1 {}, {}", dest, lhs, rhs),
            Instr::ICmp {
                dest,
                pred,
                ty,
                lhs,
                rhs,
            } => format!("{} = icmp {} {} {}, {}", dest, pred, ty.to_ll(), lhs, rhs),
            Instr::Call { dest, ret_ty, func, args } => {
                let args = args
                    .iter()
                    .map(|(t, a)| format!("{} {}", t.to_ll(), a))
                    .collect::<Vec<_>>()
                    .join(", ");
                match dest {
                    Some(d) => format!("{} = call {} @{}({})", d, ret_ty.to_ll(), func, args),
                    None => format!("call {} @{}({})", ret_ty.to_ll(), func, args),
                }
            }
            Instr::Alloca { dest, ty } => format!("{} = alloca {}", dest, ty.to_ll()),
            Instr::Store { ty, value, ptr } => {
                format!("store {} {}, {}* {}", ty.to_ll(), value, ty.to_ll(), ptr)
            }
            Instr::Load { dest, ty, ptr } => {
                format!("{} = load {}, {}* {}", dest, ty.to_ll(), ty.to_ll(), ptr)
            }
            Instr::Phi { dest, ty, incomings } => {
                let list = incomings
                    .iter()
                    .map(|(v, b)| format!("[ {}, %{} ]", v, b))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} = phi {} {}", dest, ty.to_ll(), list)
            }
        }
    }
}

impl Type {
    fn to_ll(&self) -> &'static str {
        match self {
            Type::I32 => "i32",
            Type::Bool => "i1",
            Type::Unit => "i32",
        }
    }
}

impl ToString for Terminator {
    fn to_string(&self) -> String {
        match self {
            Terminator::Ret { value } => format!("ret i32 {}", value),
            Terminator::Br { dest } => format!("br label %{}", dest),
            Terminator::CondBr {
                cond,
                then_dest,
                else_dest,
            } => format!(
                "br i1 {}, label %{}, label %{}",
                cond, then_dest, else_dest
            ),
        }
    }
}
