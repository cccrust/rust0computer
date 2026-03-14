#![allow(dead_code)]
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ModuleRef {
    pub module: Module,
}

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
pub enum Type {
    I1,
    I32,
    I64,
    Void,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Local(String),
    ConstInt(i64),
    ConstBool(bool),
}

#[derive(Debug, Clone)]
pub enum ICmpPred {
    EQ,
    NE,
    SGT,
    SGE,
    SLT,
    SLE,
    UGT,
    UGE,
    ULT,
    ULE,
}

#[derive(Debug, Clone)]
pub enum Instr {
    Add { dest: String, lhs: Operand, rhs: Operand },
    Sub { dest: String, lhs: Operand, rhs: Operand },
    Mul { dest: String, lhs: Operand, rhs: Operand },
    SDiv { dest: String, lhs: Operand, rhs: Operand },
    SRem { dest: String, lhs: Operand, rhs: Operand },
    And { dest: String, lhs: Operand, rhs: Operand },
    Or { dest: String, lhs: Operand, rhs: Operand },
    Xor { dest: String, lhs: Operand, rhs: Operand },
    ICmp { dest: String, pred: ICmpPred, lhs: Operand, rhs: Operand },
    Call { dest: Option<String>, func: String, args: Vec<Operand> },
    Alloca { dest: String },
    Store { value: Operand, ptr: Operand },
    Load { dest: String, ptr: Operand },
    Phi { dest: String, incomings: Vec<(Operand, String)> },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret { value: Operand },
    Br { dest: String },
    CondBr { cond: Operand, then_dest: String, else_dest: String },
}

impl ModuleRef {
    pub fn from_ll_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let module = parse_module(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Self { module })
    }
}

fn parse_module(text: &str) -> Result<Module> {
    let mut functions = Vec::new();
    let mut lines = text.lines().enumerate().peekable();

    while let Some((_, line)) = lines.next() {
        let line = strip_comment(line).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("define ") {
            let (func, consumed) = parse_function(line, &mut lines)?;
            functions.push(func);
            if consumed {
                continue;
            }
        }
    }

    Ok(Module { functions })
}

fn parse_function(
    first_line: String,
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>,
) -> Result<(Function, bool)> {
    let (name, params, return_type) = parse_function_signature(&first_line)?;

    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut current_block: Option<BasicBlock> = None;

    while let Some((_, raw_line)) = lines.next() {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "}" {
            if let Some(bb) = current_block.take() {
                blocks.push(bb);
            }
            return Ok((
                Function {
                    name,
                    params,
                    return_type,
                    blocks,
                },
                true,
            ));
        }

        if line.ends_with(':') {
            if let Some(bb) = current_block.take() {
                blocks.push(bb);
            }
            let label = line.trim_end_matches(':').to_string();
            current_block = Some(BasicBlock {
                name: label,
                instrs: Vec::new(),
                term: Terminator::Br {
                    dest: "__unset__".to_string(),
                },
            });
            continue;
        }

        let block = current_block
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("instruction outside of basic block"))?;

        if line.starts_with("ret ") {
            block.term = parse_ret(&line)?;
            continue;
        }
        if line.starts_with("br ") {
            block.term = parse_br(&line)?;
            continue;
        }
        if line.starts_with("store ") {
            let instr = parse_store(&line)?;
            block.instrs.push(instr);
            continue;
        }

        let instr = parse_instruction(&line)?;
        block.instrs.push(instr);
    }

    bail!("function not terminated with }}")
}

fn parse_function_signature(line: &str) -> Result<(String, Vec<Param>, Type)> {
    // define i32 @main() {
    // define i32 @add(i32 %x, i32 %y) {
    let rest = line
        .strip_prefix("define ")
        .ok_or_else(|| anyhow::anyhow!("invalid define line"))?;

    let at_pos = rest
        .find('@')
        .ok_or_else(|| anyhow::anyhow!("missing function name"))?;
    let return_ty_str = rest[..at_pos].trim();
    let return_type = parse_type(return_ty_str)?;

    let after_at = &rest[at_pos + 1..];
    let lparen = after_at
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("missing (") )?;
    let rparen = after_at
        .find(')')
        .ok_or_else(|| anyhow::anyhow!("missing )"))?;

    let name = after_at[..lparen].trim().to_string();
    let params_str = after_at[lparen + 1..rparen].trim();

    let mut params = Vec::new();
    if !params_str.is_empty() {
        for part in params_str.split(',') {
            let p = part.trim();
            let mut it = p.split_whitespace();
            let ty_str = it
                .next()
                .ok_or_else(|| anyhow::anyhow!("bad param"))?;
            let name_str = it
                .next()
                .ok_or_else(|| anyhow::anyhow!("bad param name"))?;
            let name = name_str.trim_start_matches('%').to_string();
            params.push(Param {
                name,
                ty: parse_type(ty_str)?,
            });
        }
    }

    Ok((name, params, return_type))
}

fn parse_instruction(line: &str) -> Result<Instr> {
    // %a = add i32 2, 40
    // %cond = icmp sgt i32 3, 2
    // %v = call i32 @add(i32 10, i32 32)
    let (dest, rest) = line
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("expected assignment"))?;
    let dest = dest.trim().trim_start_matches('%').to_string();
    let rest = rest.trim();

    if rest.starts_with("add ") {
        return parse_binop(rest, "add", |lhs, rhs| Instr::Add { dest, lhs, rhs });
    }
    if rest.starts_with("sub ") {
        return parse_binop(rest, "sub", |lhs, rhs| Instr::Sub { dest, lhs, rhs });
    }
    if rest.starts_with("mul ") {
        return parse_binop(rest, "mul", |lhs, rhs| Instr::Mul { dest, lhs, rhs });
    }
    if rest.starts_with("sdiv ") {
        return parse_binop(rest, "sdiv", |lhs, rhs| Instr::SDiv { dest, lhs, rhs });
    }
    if rest.starts_with("srem ") {
        return parse_binop(rest, "srem", |lhs, rhs| Instr::SRem { dest, lhs, rhs });
    }
    if rest.starts_with("and ") {
        return parse_binop(rest, "and", |lhs, rhs| Instr::And { dest, lhs, rhs });
    }
    if rest.starts_with("or ") {
        return parse_binop(rest, "or", |lhs, rhs| Instr::Or { dest, lhs, rhs });
    }
    if rest.starts_with("xor ") {
        return parse_binop(rest, "xor", |lhs, rhs| Instr::Xor { dest, lhs, rhs });
    }
    if rest.starts_with("icmp ") {
        return parse_icmp(rest, dest);
    }
    if rest.starts_with("call ") {
        return parse_call(rest, Some(dest));
    }
    if rest.starts_with("alloca ") {
        return Ok(Instr::Alloca { dest });
    }
    if rest.starts_with("load ") {
        return parse_load(rest, dest);
    }
    if rest.starts_with("phi ") {
        return parse_phi(rest, dest);
    }

    bail!("unsupported instruction: {}", line)
}

fn parse_binop<F>(rest: &str, op: &str, f: F) -> Result<Instr>
where
    F: FnOnce(Operand, Operand) -> Instr,
{
    // add i32 2, 40
    let after = rest
        .strip_prefix(op)
        .ok_or_else(|| anyhow::anyhow!("bad binop"))?
        .trim();
    let after = strip_type_prefix_any(after)?;
    let (lhs, rhs) = parse_two_operands(after)?;
    Ok(f(lhs, rhs))
}

fn parse_icmp(rest: &str, dest: String) -> Result<Instr> {
    // icmp sgt i32 3, 2
    let after = rest.strip_prefix("icmp ").unwrap().trim();
    let mut it = after.split_whitespace();
    let pred_str = it
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing icmp predicate"))?;
    let pred = parse_predicate(pred_str)?;

    let after_pred = after[pred_str.len()..].trim();
    let after_pred = strip_type_prefix_any(after_pred)?;
    let (lhs, rhs) = parse_two_operands(after_pred)?;
    Ok(Instr::ICmp { dest, pred, lhs, rhs })
}

fn parse_call(rest: &str, dest: Option<String>) -> Result<Instr> {
    // call i32 @add(i32 10, i32 32)
    let after = rest.strip_prefix("call ").unwrap().trim();
    let after = strip_type_prefix_any(after)?;

    let at_pos = after
        .find('@')
        .ok_or_else(|| anyhow::anyhow!("call missing @"))?;
    let after_at = &after[at_pos + 1..];
    let lparen = after_at
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("call missing (") )?;
    let rparen = after_at
        .rfind(')')
        .ok_or_else(|| anyhow::anyhow!("call missing )"))?;

    let func = after_at[..lparen].trim().to_string();
    let args_str = after_at[lparen + 1..rparen].trim();

    let mut args = Vec::new();
    if !args_str.is_empty() {
        for part in args_str.split(',') {
            let p = part.trim();
            let mut it = p.split_whitespace();
            let _ty = it
                .next()
                .ok_or_else(|| anyhow::anyhow!("bad call arg"))?;
            let val = it
                .next()
                .ok_or_else(|| anyhow::anyhow!("bad call arg value"))?;
            args.push(parse_operand(val)?);
        }
    }

    Ok(Instr::Call { dest, func, args })
}

fn parse_load(rest: &str, dest: String) -> Result<Instr> {
    // load i32, i32* %ptr
    let after = rest.strip_prefix("load ").unwrap().trim();
    let (_, ptr_part) = after
        .split_once(',')
        .ok_or_else(|| anyhow::anyhow!("bad load"))?;
    let ptr_part = ptr_part.trim();
    let ptr_part = ptr_part
        .strip_prefix("i32*")
        .or_else(|| ptr_part.strip_prefix("i1*"))
        .unwrap_or(ptr_part)
        .trim();
    let ptr = parse_operand(ptr_part)?;
    Ok(Instr::Load { dest, ptr })
}

fn parse_phi(rest: &str, dest: String) -> Result<Instr> {
    // phi i32 [ %a, %then ], [ %b, %else ]
    let after = rest.strip_prefix("phi ").unwrap().trim();
    let after = after
        .strip_prefix("i32")
        .or_else(|| after.strip_prefix("i1"))
        .unwrap_or(after)
        .trim();
    let mut incomings = Vec::new();
    let mut s = after;
    while let Some(start) = s.find('[') {
        let end = s[start..]
            .find(']')
            .ok_or_else(|| anyhow::anyhow!("bad phi"))?
            + start;
        let chunk = s[start + 1..end].trim();
        let mut parts = chunk.split(',');
        let val = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("bad phi incoming"))?
            .trim();
        let bb = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("bad phi incoming"))?
            .trim();
        let val = parse_operand(val)?;
        let bb = bb.trim_start_matches('%').to_string();
        incomings.push((val, bb));
        s = &s[end + 1..];
    }
    Ok(Instr::Phi { dest, incomings })
}

fn parse_ret(line: &str) -> Result<Terminator> {
    // ret i32 %a
    let after = line.strip_prefix("ret ").unwrap().trim();
    let after = strip_type_prefix(after)?;
    let value = parse_operand(after)?;
    Ok(Terminator::Ret { value })
}

fn parse_store(line: &str) -> Result<Instr> {
    // store i32 %v, i32* %ptr
    let after = line.strip_prefix("store ").unwrap().trim();
    let (val_str, rest) = after
        .split_once(',')
        .ok_or_else(|| anyhow::anyhow!("bad store"))?;
    let rest = rest.trim();
    let ptr_str = rest
        .strip_prefix("i32*")
        .or_else(|| rest.strip_prefix("i1*"))
        .unwrap_or(rest)
        .trim();
    let val_str = strip_type_prefix_any(val_str.trim())?;
    let value = parse_operand(val_str.trim())?;
    let ptr = parse_operand(ptr_str)?;
    Ok(Instr::Store { value, ptr })
}

fn parse_br(line: &str) -> Result<Terminator> {
    // br label %then
    // br i1 %cond, label %then, label %else
    let after = line.strip_prefix("br ").unwrap().trim();
    if after.starts_with("label ") {
        let dest = after
            .strip_prefix("label ")
            .unwrap()
            .trim()
            .trim_start_matches('%')
            .to_string();
        return Ok(Terminator::Br { dest });
    }

    if after.starts_with("i1 ") {
        let after = after.strip_prefix("i1 ").unwrap().trim();
        let (cond_str, rest) = after
            .split_once(',')
            .ok_or_else(|| anyhow::anyhow!("bad cond br"))?;
        let cond = parse_operand(cond_str.trim())?;

        let rest = rest.trim();
        let mut parts = rest.split(',');
        let t = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("bad cond br"))?
            .trim();
        let f = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("bad cond br"))?
            .trim();

        let then_dest = t
            .strip_prefix("label ")
            .ok_or_else(|| anyhow::anyhow!("bad true dest"))?
            .trim()
            .trim_start_matches('%')
            .to_string();
        let else_dest = f
            .strip_prefix("label ")
            .ok_or_else(|| anyhow::anyhow!("bad false dest"))?
            .trim()
            .trim_start_matches('%')
            .to_string();

        return Ok(Terminator::CondBr {
            cond,
            then_dest,
            else_dest,
        });
    }

    bail!("unsupported br: {}", line)
}

fn strip_comment(line: &str) -> &str {
    match line.find(';') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn strip_type_prefix(s: &str) -> Result<&str> {
    let mut it = s.split_whitespace();
    let ty = it.next().ok_or_else(|| anyhow::anyhow!("missing type"))?;
    let rest = s[ty.len()..].trim();
    Ok(rest)
}

fn strip_type_prefix_any(s: &str) -> Result<&str> {
    if s.starts_with("i32 ") || s.starts_with("i1 ") {
        strip_type_prefix(s)
    } else {
        Ok(s)
    }
}

fn parse_type(s: &str) -> Result<Type> {
    match s {
        "i1" => Ok(Type::I1),
        "i32" => Ok(Type::I32),
        "i64" => Ok(Type::I64),
        "void" => Ok(Type::Void),
        _ => bail!("unsupported type: {}", s),
    }
}

fn parse_two_operands(s: &str) -> Result<(Operand, Operand)> {
    let (a, b) = s
        .split_once(',')
        .ok_or_else(|| anyhow::anyhow!("expected two operands"))?;
    Ok((parse_operand(a.trim())?, parse_operand(b.trim())?))
}

fn parse_operand(s: &str) -> Result<Operand> {
    let s = s.trim();
    if s.starts_with('%') {
        return Ok(Operand::Local(s.trim_start_matches('%').to_string()));
    }
    if s == "true" {
        return Ok(Operand::ConstBool(true));
    }
    if s == "false" {
        return Ok(Operand::ConstBool(false));
    }
    let val = s.parse::<i64>().with_context(|| format!("bad int: {}", s))?;
    Ok(Operand::ConstInt(val))
}

fn parse_predicate(s: &str) -> Result<ICmpPred> {
    match s {
        "eq" => Ok(ICmpPred::EQ),
        "ne" => Ok(ICmpPred::NE),
        "sgt" => Ok(ICmpPred::SGT),
        "sge" => Ok(ICmpPred::SGE),
        "slt" => Ok(ICmpPred::SLT),
        "sle" => Ok(ICmpPred::SLE),
        "ugt" => Ok(ICmpPred::UGT),
        "uge" => Ok(ICmpPred::UGE),
        "ult" => Ok(ICmpPred::ULT),
        "ule" => Ok(ICmpPred::ULE),
        _ => bail!("unsupported icmp predicate: {}", s),
    }
}
