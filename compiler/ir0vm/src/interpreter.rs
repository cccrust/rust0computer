use crate::ir::{ICmpPred, Instr, Operand, Terminator};
use crate::linker::{FunctionRef, LinkedProgram};
use crate::memory::Memory;
use crate::value::Value;
use crate::vm::Vm;
use anyhow::{bail, Result};
use std::collections::HashMap;

pub struct Interpreter<'a> {
    vm: &'a mut Vm,
}

impl<'a> Interpreter<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        Self { vm }
    }

    pub fn run_main(&mut self) -> Result<i32> {
        let main_ref = self
            .vm
            .program
            .functions
            .get("main")
            .copied()
            .ok_or_else(|| anyhow::anyhow!("main function not found"))?;

        let value = self.run_function(main_ref, vec![])?;
        value
            .as_i32()
            .ok_or_else(|| anyhow::anyhow!("main did not return i32"))
    }

    fn run_function(&mut self, func_ref: FunctionRef, args: Vec<Value>) -> Result<Value> {
        let program: &LinkedProgram = &self.vm.program;
        let module = &program.modules[func_ref.module_index].module;
        let function = module.functions[func_ref.function_index].clone();

        if function.params.len() != args.len() {
            bail!(
                "argument count mismatch for {}: expected {}, got {}",
                function.name,
                function.params.len(),
                args.len()
            );
        }

        let mut memory = Memory::default();
        for (param, value) in function.params.iter().zip(args.into_iter()) {
            memory.set_local(&param.name, value);
        }

        let mut bb_index = HashMap::new();
        for (idx, bb) in function.blocks.iter().enumerate() {
            bb_index.insert(bb.name.clone(), idx);
        }

        let mut current_bb = 0usize;
        let mut prev_bb_name: Option<String> = None;
        loop {
            let bb = &function.blocks[current_bb];

            for instr in &bb.instrs {
                if let crate::ir::Instr::Phi { .. } = instr {
                    self.exec_phi(instr, &mut memory, prev_bb_name.as_deref())?;
                }
            }
            for instr in &bb.instrs {
                if let crate::ir::Instr::Phi { .. } = instr {
                    continue;
                }
                self.exec_instruction(instr, &mut memory)?;
            }

            match &bb.term {
                Terminator::Ret { value } => {
                    let v = self.eval_operand(value, &memory)?;
                    return Ok(v);
                }
                Terminator::Br { dest } => {
                    prev_bb_name = Some(bb.name.clone());
                    current_bb = *bb_index
                        .get(dest)
                        .ok_or_else(|| anyhow::anyhow!("unknown basic block {}", dest))?;
                }
                Terminator::CondBr {
                    cond,
                    then_dest,
                    else_dest,
                } => {
                    let cond = self
                        .eval_operand(cond, &memory)?
                        .as_i1()
                        .ok_or_else(|| anyhow::anyhow!("branch condition is not i1"))?;
                    let dest = if cond { then_dest } else { else_dest };
                    prev_bb_name = Some(bb.name.clone());
                    current_bb = *bb_index
                        .get(dest)
                        .ok_or_else(|| anyhow::anyhow!("unknown basic block {}", dest))?;
                }
            }
        }
    }

    fn exec_instruction(&mut self, instr: &Instr, memory: &mut Memory) -> Result<()> {
        match instr {
            Instr::Add { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("add expects i32 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("add expects i32 operands")
                })?;
                memory.set_local(dest, Value::I32(lhs.wrapping_add(rhs)));
            }
            Instr::Sub { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("sub expects i32 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("sub expects i32 operands")
                })?;
                memory.set_local(dest, Value::I32(lhs.wrapping_sub(rhs)));
            }
            Instr::Mul { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("mul expects i32 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("mul expects i32 operands")
                })?;
                memory.set_local(dest, Value::I32(lhs.wrapping_mul(rhs)));
            }
            Instr::SDiv { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("sdiv expects i32 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("sdiv expects i32 operands")
                })?;
                if rhs == 0 {
                    bail!("division by zero");
                }
                memory.set_local(dest, Value::I32(lhs / rhs));
            }
            Instr::SRem { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("srem expects i32 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i32().ok_or_else(|| {
                    anyhow::anyhow!("srem expects i32 operands")
                })?;
                if rhs == 0 {
                    bail!("division by zero");
                }
                memory.set_local(dest, Value::I32(lhs % rhs));
            }
            Instr::And { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("and expects i1 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("and expects i1 operands")
                })?;
                memory.set_local(dest, Value::I1(lhs & rhs));
            }
            Instr::Or { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("or expects i1 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("or expects i1 operands")
                })?;
                memory.set_local(dest, Value::I1(lhs | rhs));
            }
            Instr::Xor { dest, lhs, rhs } => {
                let lhs = self.eval_operand(lhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("xor expects i1 operands")
                })?;
                let rhs = self.eval_operand(rhs, memory)?.as_i1().ok_or_else(|| {
                    anyhow::anyhow!("xor expects i1 operands")
                })?;
                memory.set_local(dest, Value::I1(lhs ^ rhs));
            }
            Instr::ICmp {
                dest,
                pred,
                lhs,
                rhs,
            } => {
                let lhs_val = self.eval_operand(lhs, memory)?;
                let rhs_val = self.eval_operand(rhs, memory)?;
                let result = match (lhs_val, rhs_val) {
                    (Value::I32(l), Value::I32(r)) => self.eval_icmp(pred, l, r),
                    (Value::I1(l), Value::I1(r)) => match pred {
                        crate::ir::ICmpPred::EQ => l == r,
                        crate::ir::ICmpPred::NE => l != r,
                        _ => bail!("icmp predicate not supported for i1"),
                    },
                    _ => bail!("icmp operand type mismatch"),
                };
                memory.set_local(dest, Value::I1(result));
            }
            Instr::Call { dest, func, args } => {
                let mut values = Vec::new();
                for op in args {
                    values.push(self.eval_operand(op, memory)?);
                }

                let func_ref = self
                    .vm
                    .program
                    .functions
                    .get(func)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("unknown function {}", func))?;

                let ret = self.run_function(func_ref, values)?;
                if let Some(dest) = dest {
                    memory.set_local(dest, ret);
                }
            }
            Instr::Alloca { dest } => {
                memory.alloc_slot(dest);
                memory.set_local(dest, Value::Ptr(dest.clone()));
            }
            Instr::Store { value, ptr } => {
                let val = self.eval_operand(value, memory)?;
                let ptr_val = self.eval_operand(ptr, memory)?;
                let name = match ptr_val {
                    Value::Ptr(n) => n,
                    _ => bail!("store expects pointer"),
                };
                if !memory.store_slot(&name, val) {
                    bail!("store to unknown slot {}", name);
                }
            }
            Instr::Load { dest, ptr } => {
                let ptr_val = self.eval_operand(ptr, memory)?;
                let name = match ptr_val {
                    Value::Ptr(n) => n,
                    _ => bail!("load expects pointer"),
                };
                let val = memory
                    .load_slot(&name)
                    .ok_or_else(|| anyhow::anyhow!("load from unknown slot {}", name))?;
                memory.set_local(dest, val);
            }
            Instr::Phi { .. } => {
                // handled before regular instructions
            }
        }

        Ok(())
    }

    fn exec_phi(
        &mut self,
        instr: &crate::ir::Instr,
        memory: &mut Memory,
        pred: Option<&str>,
    ) -> Result<()> {
        let pred = pred.ok_or_else(|| anyhow::anyhow!("phi in entry block"))?;
        if let crate::ir::Instr::Phi { dest, incomings } = instr {
            for (op, bb) in incomings {
                if bb == pred {
                    let v = self.eval_operand(op, memory)?;
                    memory.set_local(dest, v);
                    return Ok(());
                }
            }
            bail!("phi has no incoming for pred {}", pred)
        } else {
            Ok(())
        }
    }

    fn eval_operand(&self, op: &Operand, memory: &Memory) -> Result<Value> {
        match op {
            Operand::Local(name) => memory
                .get_local(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown local {}", name)),
            Operand::ConstInt(v) => {
                let v = i32::try_from(*v)
                    .map_err(|_| anyhow::anyhow!("const int out of i32 range"))?;
                Ok(Value::I32(v))
            }
            Operand::ConstBool(b) => Ok(Value::I1(*b)),
        }
    }

    fn eval_icmp(&self, pred: &ICmpPred, lhs: i32, rhs: i32) -> bool {
        match pred {
            ICmpPred::EQ => lhs == rhs,
            ICmpPred::NE => lhs != rhs,
            ICmpPred::SGT => lhs > rhs,
            ICmpPred::SGE => lhs >= rhs,
            ICmpPred::SLT => lhs < rhs,
            ICmpPred::SLE => lhs <= rhs,
            ICmpPred::UGT => (lhs as u32) > (rhs as u32),
            ICmpPred::UGE => (lhs as u32) >= (rhs as u32),
            ICmpPred::ULT => (lhs as u32) < (rhs as u32),
            ICmpPred::ULE => (lhs as u32) <= (rhs as u32),
        }
    }
}
