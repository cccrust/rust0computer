/// VVP Code Generator
/// Generates VVP (Verilog VPI Procedural) bytecode compatible with the `vvp` runtime.
/// VVP is the bytecode format used by Icarus Verilog (iverilog).
///
/// VVP file format:
///   Line 1: `:ivl_version "..."` or `#!/usr/bin/vvp`
///   Followed by functor/net declarations and thread code.

use crate::elaborator::*;
use crate::parser::*;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

#[derive(Debug)]
pub struct CodeGenError {
    pub msg: String,
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGen error: {}", self.msg)
    }
}

pub fn generate(design: &Design) -> Result<String, CodeGenError> {
    let mut gen = VvpGen::new();
    gen.generate(design)
}

// ============================================================
// VVP Instruction / Functor definitions
// ============================================================

struct VvpGen {
    out: String,
    label_counter: usize,
    var_counter: usize,
    // Signal name -> VVP net label
    net_map: HashMap<String, String>,
    // Temp register counter
    tmp_counter: usize,
}

impl VvpGen {
    fn new() -> Self {
        VvpGen {
            out: String::new(),
            label_counter: 0,
            var_counter: 0,
            net_map: HashMap::new(),
            tmp_counter: 0,
        }
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let l = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        l
    }

    fn fresh_var(&mut self) -> String {
        let v = format!("T{}", self.var_counter);
        self.var_counter += 1;
        v
    }

    fn fresh_tmp(&mut self) -> String {
        let t = format!("_tmp_{}", self.tmp_counter);
        self.tmp_counter += 1;
        t
    }

    fn emit(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn emitf(&mut self, s: String) {
        self.out.push_str(&s);
        self.out.push('\n');
    }

    // ---- Top-level generation ----

    fn generate(&mut self, design: &Design) -> Result<String, CodeGenError> {
        // VVP file header
        self.emit("#!/usr/bin/vvp");
        self.emitf(format!(":ivl_version \"0.1\" \"Rust Verilog Compiler (rverilog)\";"));
        self.emitf(format!(":ivl_delay_selection \"TYPICAL\";"));
        self.emit(":vpi_time_precision + 0;");
        self.emit("");

        // Generate the top module (and recursively referenced ones)
        let top = &design.top_module;
        if let Some(module) = design.modules.get(top) {
            self.gen_module(module, design)?;
        }

        // Entry point: initial thread that runs $finish
        self.emit("");
        self.emit(":vpi_module \"system.vpi\";");
        self.emit("");

        Ok(self.out.clone())
    }

    fn gen_module(&mut self, module: &ElabModule, design: &Design) -> Result<(), CodeGenError> {
        let mod_prefix = format!("v{}", module.name);

        self.emitf(format!("# Module: {}", module.name));
        self.emitf(format!("# Ports: {}", module.ports.iter()
            .map(|p| format!("{:?} {}", p.dir, p.name))
            .collect::<Vec<_>>()
            .join(", ")));
        self.emit("");

        // Declare all signals as VVP nets/variables
        for (sig_name, sig) in &module.signals {
            let label = format!("{}/{}", mod_prefix, sig_name);
            self.net_map.insert(sig_name.clone(), label.clone());

            match &sig.kind {
                SignalKind::Wire => {
                    // .net is a wire (combinational)
                    self.emitf(format!("{} .net \"{}\" {}, {}; #{} wire",
                        label, sig_name, sig.msb, sig.lsb, sig.width));
                }
                SignalKind::Reg => {
                    // .var is a reg (storage)
                    self.emitf(format!("{} .var \"{}\" {}, {};",
                        label, sig_name, sig.msb, sig.lsb));
                }
                SignalKind::Integer => {
                    self.emitf(format!("{} .var/i \"{}\" 31, 0;",
                        label, sig_name));
                }
                SignalKind::Real => {
                    self.emitf(format!("{} .var/real \"{}\";", label, sig_name));
                }
                SignalKind::Parameter(val) => {
                    self.emitf(format!("# parameter {} = {}", sig_name, val));
                }
            }
        }
        self.emit("");

        // Continuous assignments -> functors
        for (idx, ca) in module.continuous_assigns.iter().enumerate() {
            let lhs_label = self.get_net_label(&ca.lhs);
            let rhs_label = self.gen_expr_functor(&ca.rhs, &mod_prefix, idx)?;
            self.emitf(format!("{}  .assign/net {}, 0, 0, {};",
                lhs_label, rhs_label, lhs_label));
        }
        self.emit("");

        // Gate instances
        for gate in &module.gate_instances {
            self.gen_gate(gate, &mod_prefix)?;
        }
        self.emit("");

        // Module instances (hierarchical)
        for inst in &module.instances {
            self.emitf(format!("# Instance: {} of {}", inst.inst_name, inst.module_name));
            if let Some(sub_mod) = design.modules.get(&inst.module_name) {
                // Recursively generate if not already done
                // (simplified: just emit a comment for now)
                self.emitf(format!("# (instantiation of module {})", inst.module_name));
                let _ = sub_mod; // suppress unused warning
            }
        }

        // Always blocks -> process threads
        for (idx, always) in module.always_blocks.iter().enumerate() {
            let thread_label = format!("{}_always_{}", mod_prefix, idx);
            self.gen_always_thread(&thread_label, always, &mod_prefix)?;
        }
        self.emit("");

        // Initial blocks -> threads
        for (idx, initial) in module.initial_blocks.iter().enumerate() {
            let thread_label = format!("{}_initial_{}", mod_prefix, idx);
            self.gen_initial_thread(&thread_label, initial, &mod_prefix)?;
        }

        Ok(())
    }

    // ---- Net / Signal helpers ----

    fn get_net_label(&self, expr: &ElabExpr) -> String {
        match expr {
            ElabExpr::Signal(name, _) => {
                self.net_map.get(name).cloned().unwrap_or_else(|| name.clone())
            }
            _ => "_unknown_net_".to_string(),
        }
    }

    fn signal_label(&self, name: &str) -> String {
        self.net_map.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    // ---- Expression -> functor (for continuous assign / gate) ----

    fn gen_expr_functor(&mut self, expr: &ElabExpr, prefix: &str, idx: usize)
        -> Result<String, CodeGenError>
    {
        match expr {
            ElabExpr::Signal(name, _) => {
                Ok(self.signal_label(name))
            }
            ElabExpr::IntLit(val, width) => {
                let label = format!("{}_const_{}_{}", prefix, idx, val);
                let bits = format_bits(*val, *width);
                self.emitf(format!("{} .net/const \"{}\", 0, {};",
                    label, bits, width - 1));
                Ok(label)
            }
            ElabExpr::Binary(op, lhs, rhs, width) => {
                let ll = self.gen_expr_functor(lhs, prefix, idx * 100)?;
                let rl = self.gen_expr_functor(rhs, prefix, idx * 100 + 1)?;
                let label = format!("{}_binop_{}_{}", prefix, idx, self.fresh_label("b"));
                let functor = match op {
                    BinaryOp::BitAnd => format!("{} .and/{} {}, {};", label, width, ll, rl),
                    BinaryOp::BitOr  => format!("{} .or/{} {}, {};",  label, width, ll, rl),
                    BinaryOp::BitXor => format!("{} .xor/{} {}, {};", label, width, ll, rl),
                    BinaryOp::Add    => format!("{} .arith/sum {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Sub    => format!("{} .arith/sub {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Mul    => format!("{} .arith/mult {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Eq     => format!("{} .cmp/eq {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Ne     => format!("{} .cmp/ne {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Lt     => format!("{} .cmp/lt {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Gt     => format!("{} .cmp/gt {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Le     => format!("{} .cmp/le {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Ge     => format!("{} .cmp/ge {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Shl    => format!("{} .shift/l {}, {}, {};", label, width, ll, rl),
                    BinaryOp::Shr    => format!("{} .shift/r {}, {}, {};", label, width, ll, rl),
                    _ => format!("{} .and/{} {}, {}; # unsupported op {:?}", label, width, ll, rl, op),
                };
                self.emitf(functor);
                Ok(label)
            }
            ElabExpr::Unary(op, e, width) => {
                let el = self.gen_expr_functor(e, prefix, idx)?;
                let label = format!("{}_unop_{}_{}", prefix, idx, self.fresh_label("u"));
                let functor = match op {
                    UnaryOp::BitNot => format!("{} .inv/{} {};", label, width, el),
                    UnaryOp::RedAnd => format!("{} .reduce/and/{} {};", label, width, el),
                    UnaryOp::RedOr  => format!("{} .reduce/or/{} {};",  label, width, el),
                    UnaryOp::RedXor => format!("{} .reduce/xor/{} {};", label, width, el),
                    _ => format!("{} .inv/{} {}; # {:?}", label, width, el, op),
                };
                self.emitf(functor);
                Ok(label)
            }
            ElabExpr::Concat(parts, width) => {
                let labels: Result<Vec<_>, _> = parts.iter().enumerate()
                    .map(|(i, p)| self.gen_expr_functor(p, prefix, idx * 100 + i))
                    .collect();
                let labels = labels?;
                let label = format!("{}_concat_{}", prefix, self.fresh_label("c"));
                let parts_str = labels.join(", ");
                self.emitf(format!("{} .concat/{} {};", label, width, parts_str));
                Ok(label)
            }
            ElabExpr::Index(base, idx_expr) => {
                let bl = self.gen_expr_functor(base, prefix, idx)?;
                // Simplified: just reference the net with subscript
                let label = format!("{}_idx_{}", prefix, self.fresh_label("i"));
                self.emitf(format!("{} .part {} /* index */;", label, bl));
                Ok(label)
            }
            ElabExpr::Ternary(cond, then_, else_, width) => {
                let cl = self.gen_expr_functor(cond, prefix, idx)?;
                let tl = self.gen_expr_functor(then_, prefix, idx * 100)?;
                let el = self.gen_expr_functor(else_, prefix, idx * 100 + 1)?;
                let label = format!("{}_mux_{}", prefix, self.fresh_label("m"));
                self.emitf(format!("{} .mux/{} {}, {}, {};", label, width, cl, tl, el));
                Ok(label)
            }
            _ => {
                // Fallback: emit a constant 0
                let label = format!("{}_zero_{}", prefix, self.fresh_label("z"));
                self.emitf(format!("{} .net/const \"0\", 0, 0;", label));
                Ok(label)
            }
        }
    }

    // ---- Gate primitive generation ----

    fn gen_gate(&mut self, gate: &ElabGate, prefix: &str) -> Result<(), CodeGenError> {
        let out_label = self.get_net_label(&gate.output);
        let inp_labels: Result<Vec<_>, _> = gate.inputs.iter().enumerate()
            .map(|(i, inp)| self.gen_expr_functor(inp, prefix, i))
            .collect();
        let inp_labels = inp_labels?;
        let inp_str = inp_labels.join(", ");

        let gate_fn = match gate.gate_type {
            GateType::And  => "and",
            GateType::Or   => "or",
            GateType::Not  => "not",
            GateType::Nand => "nand",
            GateType::Nor  => "nor",
            GateType::Xor  => "xor",
            GateType::Xnor => "xnor",
            GateType::Buf  => "buf",
        };
        self.emitf(format!("{} .{}/1 {};", out_label, gate_fn, inp_str));
        Ok(())
    }

    // ---- Always block -> VVP thread ----

    fn gen_always_thread(&mut self, label: &str, always: &ElabAlways, prefix: &str)
        -> Result<(), CodeGenError>
    {
        self.emitf(format!("# always block"));
        self.emitf(format!("{} .thread @always_body_{};", label, label));
        self.emitf(format!("@always_body_{}:", label));

        // Sensitivity
        match &always.sensitivity {
            Sensitivity::Star => {
                // @(*) -> combinational: re-evaluate on any input change
                // In VVP this is modeled as: wait for any change
                self.emitf(format!("    %wait/event @{}__comb;", label));
            }
            Sensitivity::List(events) => {
                // Wait for edge events
                for (i, ev) in events.iter().enumerate() {
                    let sig_label = match &ev.signal {
                        Expr::Ident(name) => self.signal_label(name),
                        _ => "_sig_".to_string(),
                    };
                    let edge_flag = match &ev.edge {
                        Some(Edge::Posedge) => "posedge",
                        Some(Edge::Negedge) => "negedge",
                        None => "anyedge",
                    };
                    self.emitf(format!("    # wait {} {}", edge_flag, sig_label));
                    self.emitf(format!("    %wait/{} {};", edge_flag, sig_label));
                    if i + 1 < events.len() {
                        self.emitf(format!("    %or;"));
                    }
                }
            }
            Sensitivity::None => {
                // no sensitivity: runs once
            }
        }

        // Body
        self.gen_stmt_vvp(&always.body, prefix, label)?;

        // Loop back for always
        self.emitf(format!("    %jmp @always_body_{};", label));
        self.emit("");
        Ok(())
    }

    fn gen_initial_thread(&mut self, label: &str, initial: &ElabInitial, prefix: &str)
        -> Result<(), CodeGenError>
    {
        self.emitf(format!("# initial block"));
        self.emitf(format!("{} .thread @initial_body_{};", label, label));
        self.emitf(format!("@initial_body_{}:", label));

        self.gen_stmt_vvp(&initial.body, prefix, label)?;

        self.emitf(format!("    %end;"));
        self.emit("");
        Ok(())
    }

    // ---- Statement -> VVP instructions ----

    fn gen_stmt_vvp(&mut self, stmt: &ElabStmt, prefix: &str, thread: &str)
        -> Result<(), CodeGenError>
    {
        match stmt {
            ElabStmt::Block(stmts) => {
                for s in stmts {
                    self.gen_stmt_vvp(s, prefix, thread)?;
                }
            }
            ElabStmt::ForkJoin(stmts) => {
                // Simplified: sequential execution (true fork/join needs threading)
                self.emitf(format!("    # fork-join (simplified as sequential)"));
                for s in stmts {
                    self.gen_stmt_vvp(s, prefix, thread)?;
                }
            }
            ElabStmt::If(cond, then_, else_) => {
                let cond_reg = self.fresh_var();
                self.gen_expr_to_reg(cond, &cond_reg, prefix)?;
                let else_label = self.fresh_label("else");
                let end_label = self.fresh_label("endif");

                self.emitf(format!("    %jmp/false {}, @{};", cond_reg, else_label));
                self.gen_stmt_vvp(then_, prefix, thread)?;
                self.emitf(format!("    %jmp @{};", end_label));
                self.emitf(format!("@{}:", else_label));
                if let Some(else_stmt) = else_ {
                    self.gen_stmt_vvp(else_stmt, prefix, thread)?;
                }
                self.emitf(format!("@{}:", end_label));
            }
            ElabStmt::Case(expr, _kind, items, default) => {
                let expr_reg = self.fresh_var();
                self.gen_expr_to_reg(expr, &expr_reg, prefix)?;
                let end_label = self.fresh_label("endcase");

                for (patterns, stmt) in items {
                    let body_label = self.fresh_label("case_body");
                    let skip_label = self.fresh_label("case_skip");

                    for pat in patterns {
                        let pat_reg = self.fresh_var();
                        self.gen_expr_to_reg(pat, &pat_reg, prefix)?;
                        self.emitf(format!("    %cmp/eq {}, {};", expr_reg, pat_reg));
                        self.emitf(format!("    %jmp/true 4, @{};", body_label));
                    }
                    self.emitf(format!("    %jmp @{};", skip_label));
                    self.emitf(format!("@{}:", body_label));
                    self.gen_stmt_vvp(stmt, prefix, thread)?;
                    self.emitf(format!("    %jmp @{};", end_label));
                    self.emitf(format!("@{}:", skip_label));
                }
                if let Some(def) = default {
                    self.gen_stmt_vvp(def, prefix, thread)?;
                }
                self.emitf(format!("@{}:", end_label));
            }
            ElabStmt::For(init, cond, step, body) => {
                let cond_label = self.fresh_label("for_cond");
                let end_label = self.fresh_label("for_end");
                self.gen_stmt_vvp(init, prefix, thread)?;
                self.emitf(format!("@{}:", cond_label));
                let cond_reg = self.fresh_var();
                self.gen_expr_to_reg(cond, &cond_reg, prefix)?;
                self.emitf(format!("    %jmp/false {}, @{};", cond_reg, end_label));
                self.gen_stmt_vvp(body, prefix, thread)?;
                self.gen_stmt_vvp(step, prefix, thread)?;
                self.emitf(format!("    %jmp @{};", cond_label));
                self.emitf(format!("@{}:", end_label));
            }
            ElabStmt::While(cond, body) => {
                let cond_label = self.fresh_label("while_cond");
                let end_label = self.fresh_label("while_end");
                self.emitf(format!("@{}:", cond_label));
                let cond_reg = self.fresh_var();
                self.gen_expr_to_reg(cond, &cond_reg, prefix)?;
                self.emitf(format!("    %jmp/false {}, @{};", cond_reg, end_label));
                self.gen_stmt_vvp(body, prefix, thread)?;
                self.emitf(format!("    %jmp @{};", cond_label));
                self.emitf(format!("@{}:", end_label));
            }
            ElabStmt::Repeat(count, body) => {
                let cnt_reg = self.fresh_var();
                let loop_label = self.fresh_label("repeat");
                let end_label = self.fresh_label("repeat_end");
                self.gen_expr_to_reg(count, &cnt_reg, prefix)?;
                self.emitf(format!("@{}:", loop_label));
                self.emitf(format!("    %jmp/false {}, @{};", cnt_reg, end_label));
                self.gen_stmt_vvp(body, prefix, thread)?;
                self.emitf(format!("    %sub/wr {}, 1;", cnt_reg));
                self.emitf(format!("    %jmp @{};", loop_label));
                self.emitf(format!("@{}:", end_label));
            }
            ElabStmt::Forever(body) => {
                let loop_label = self.fresh_label("forever");
                self.emitf(format!("@{}:", loop_label));
                self.gen_stmt_vvp(body, prefix, thread)?;
                self.emitf(format!("    %jmp @{};", loop_label));
            }
            ElabStmt::BlockingAssign(lhs, rhs) => {
                let rhs_reg = self.fresh_var();
                self.gen_expr_to_reg(rhs, &rhs_reg, prefix)?;
                let lhs_label = self.get_net_label(lhs);
                // %assign/wr for blocking assignment (immediate)
                self.emitf(format!("    %assign/wr {}, 0, {};", lhs_label, rhs_reg));
            }
            ElabStmt::NonBlockingAssign(lhs, rhs) => {
                let rhs_reg = self.fresh_var();
                self.gen_expr_to_reg(rhs, &rhs_reg, prefix)?;
                let lhs_label = self.get_net_label(lhs);
                // %assign/nb for non-blocking (end-of-timestep)
                self.emitf(format!("    %assign/nb {}, 0, {};", lhs_label, rhs_reg));
            }
            ElabStmt::EventControl(sens, body) => {
                match sens {
                    Sensitivity::Star => {
                        self.emitf(format!("    # @(*) event control"));
                    }
                    Sensitivity::List(events) => {
                        for ev in events {
                            let sig_label = match &ev.signal {
                                Expr::Ident(name) => self.signal_label(name),
                                _ => "_sig_".to_string(),
                            };
                            let edge = match &ev.edge {
                                Some(Edge::Posedge) => "posedge",
                                Some(Edge::Negedge) => "negedge",
                                None => "anyedge",
                            };
                            self.emitf(format!("    %wait/{} {};", edge, sig_label));
                        }
                    }
                    Sensitivity::None => {}
                }
                self.gen_stmt_vvp(body, prefix, thread)?;
            }
            ElabStmt::DelayControl(delay, body) => {
                let delay_reg = self.fresh_var();
                self.gen_expr_to_reg(delay, &delay_reg, prefix)?;
                self.emitf(format!("    %delay {};", delay_reg));
                self.gen_stmt_vvp(body, prefix, thread)?;
            }
            ElabStmt::SystemTask(name, args) => {
                self.gen_system_task(name, args, prefix)?;
            }
            ElabStmt::TaskCall(name, args) => {
                let args_str: Vec<String> = args.iter().enumerate()
                    .map(|(i, _a)| {
                        let r = format!("_arg_{}", i);
                        r
                    })
                    .collect();
                for (i, arg) in args.iter().enumerate() {
                    let r = format!("_arg_{}", i);
                    let _ = self.gen_expr_to_reg(arg, &r, prefix);
                }
                self.emitf(format!("    %call/task \"{}\", {};", name, args_str.join(", ")));
            }
            ElabStmt::Disable(name) => {
                self.emitf(format!("    %disable \"{}\";", name));
            }
            ElabStmt::Null => {
                self.emitf(format!("    # null statement"));
            }
        }
        Ok(())
    }

    // ---- System task generation ----

    fn gen_system_task(&mut self, name: &str, args: &[ElabExpr], prefix: &str)
        -> Result<(), CodeGenError>
    {
        match name {
            "$display" | "$write" | "$strobe" | "$monitor" => {
                self.emitf(format!("    %vpi_call \"{}\"", name));
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        ElabExpr::StringLit(s) => {
                            self.emitf(format!("       , \"{}\"", s.replace('"', "\\\"")));
                        }
                        _ => {
                            let reg = self.fresh_var();
                            self.gen_expr_to_reg(arg, &reg, prefix)?;
                            self.emitf(format!("       , {}", reg));
                        }
                    }
                    if i + 1 == args.len() {
                        self.emitf(format!("       ;"));
                    }
                }
                if args.is_empty() {
                    self.emitf(format!("       ;"));
                }
            }
            "$finish" => {
                let code = if args.is_empty() { 0 } else {
                    match &args[0] { ElabExpr::IntLit(n, _) => *n as i32, _ => 0 }
                };
                self.emitf(format!("    %vpi_call \"$finish\" , {};", code));
                self.emitf(format!("    %end;"));
            }
            "$stop" => {
                self.emitf(format!("    %vpi_call \"$stop\" ;"));
            }
            "$time" | "$realtime" => {
                self.emitf(format!("    %vpi_call \"{}\";", name));
            }
            "$dumpvars" | "$dumpfile" | "$dumpall" | "$dumpflush" => {
                self.emitf(format!("    %vpi_call \"{}\"", name));
                for arg in args {
                    match arg {
                        ElabExpr::StringLit(s) => {
                            self.emitf(format!("        , \"{}\"", s));
                        }
                        _ => {
                            let reg = self.fresh_var();
                            self.gen_expr_to_reg(arg, &reg, prefix)?;
                            self.emitf(format!("        , {}", reg));
                        }
                    }
                }
                self.emitf(format!("        ;"));
            }
            "$random" => {
                self.emitf(format!("    %vpi_call \"$random\";"));
            }
            "$readmemb" | "$readmemh" => {
                self.emitf(format!("    %vpi_call \"{}\" ;", name));
            }
            _ => {
                self.emitf(format!("    %vpi_call \"{}\" ;  # unknown system task", name));
            }
        }
        Ok(())
    }

    // ---- Expression -> register ----

    fn gen_expr_to_reg(&mut self, expr: &ElabExpr, reg: &str, prefix: &str)
        -> Result<(), CodeGenError>
    {
        match expr {
            ElabExpr::IntLit(val, width) => {
                self.emitf(format!("    %pushi {}, {}, {}; # load const", reg, val, width));
            }
            ElabExpr::RealLit(f) => {
                self.emitf(format!("    %pushf {}, {};", reg, f));
            }
            ElabExpr::StringLit(s) => {
                self.emitf(format!("    %pushs {}, \"{}\";", reg, s.replace('"', "\\\"")));
            }
            ElabExpr::Signal(name, _) => {
                let label = self.signal_label(name);
                self.emitf(format!("    %load/vec4 {}, {};", reg, label));
            }
            ElabExpr::Index(base, idx) => {
                let base_label = self.get_net_label(base);
                let idx_reg = self.fresh_var();
                self.gen_expr_to_reg(idx, &idx_reg, prefix)?;
                self.emitf(format!("    %load/vec4/p {}, {}, {};", reg, base_label, idx_reg));
            }
            ElabExpr::Slice(base, msb, lsb) => {
                let base_label = self.get_net_label(base);
                let msb_reg = self.fresh_var();
                let lsb_reg = self.fresh_var();
                self.gen_expr_to_reg(msb, &msb_reg, prefix)?;
                self.gen_expr_to_reg(lsb, &lsb_reg, prefix)?;
                self.emitf(format!("    %load/vec4/s {}, {}, {}, {};",
                    reg, base_label, msb_reg, lsb_reg));
            }
            ElabExpr::Unary(op, operand, _) => {
                let op_reg = self.fresh_var();
                self.gen_expr_to_reg(operand, &op_reg, prefix)?;
                let instr = match op {
                    UnaryOp::Minus  => format!("    %negate {}, {};", reg, op_reg),
                    UnaryOp::BitNot => format!("    %inv {}, {};", reg, op_reg),
                    UnaryOp::Not    => format!("    %cmp/z {}, {};", reg, op_reg),
                    UnaryOp::RedAnd => format!("    %and/r {}, {};", reg, op_reg),
                    UnaryOp::RedOr  => format!("    %or/r {}, {};", reg, op_reg),
                    UnaryOp::RedXor => format!("    %xor/r {}, {};", reg, op_reg),
                    _ => format!("    %mov {}, {};", reg, op_reg),
                };
                self.emitf(instr);
            }
            ElabExpr::Binary(op, lhs, rhs, _) => {
                let lreg = self.fresh_var();
                let rreg = self.fresh_var();
                self.gen_expr_to_reg(lhs, &lreg, prefix)?;
                self.gen_expr_to_reg(rhs, &rreg, prefix)?;
                let instr = match op {
                    BinaryOp::Add    => format!("    %add {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Sub    => format!("    %sub {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Mul    => format!("    %mul {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Div    => format!("    %div {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Mod    => format!("    %mod {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::BitAnd => format!("    %and {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::BitOr  => format!("    %or  {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::BitXor => format!("    %xor {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::BitXnor=> format!("    %xnor {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Eq     => format!("    %cmp/eq {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Ne     => format!("    %cmp/ne {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Lt     => format!("    %cmp/lt {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Le     => format!("    %cmp/le {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Gt     => format!("    %cmp/gt {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Ge     => format!("    %cmp/ge {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::LogAnd => format!("    %land {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::LogOr  => format!("    %lor  {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Shl    => format!("    %shiftl {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::Shr    => format!("    %shiftr {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::AShl   => format!("    %ashiftl {}, {}, {};", reg, lreg, rreg),
                    BinaryOp::AShr   => format!("    %ashiftr {}, {}, {};", reg, lreg, rreg),
                    _ => format!("    %add {}, {}, {}; # fallback", reg, lreg, rreg),
                };
                self.emitf(instr);
            }
            ElabExpr::Ternary(cond, then_, else_, _) => {
                let cond_reg = self.fresh_var();
                self.gen_expr_to_reg(cond, &cond_reg, prefix)?;
                let else_label = self.fresh_label("ternary_else");
                let end_label = self.fresh_label("ternary_end");
                self.emitf(format!("    %jmp/false {}, @{};", cond_reg, else_label));
                self.gen_expr_to_reg(then_, reg, prefix)?;
                self.emitf(format!("    %jmp @{};", end_label));
                self.emitf(format!("@{}:", else_label));
                self.gen_expr_to_reg(else_, reg, prefix)?;
                self.emitf(format!("@{}:", end_label));
            }
            ElabExpr::Concat(parts, _) => {
                let part_regs: Result<Vec<_>, _> = parts.iter().map(|p| {
                    let r = self.fresh_var();
                    self.gen_expr_to_reg(p, &r, prefix)?;
                    Ok::<_, CodeGenError>(r)
                }).collect();
                let part_regs = part_regs?;
                // Build concat from MSB to LSB
                self.emitf(format!("    %concat {}, {};", reg, part_regs.join(", ")));
            }
            ElabExpr::Repeat(count, parts, _) => {
                let count_reg = self.fresh_var();
                self.gen_expr_to_reg(count, &count_reg, prefix)?;
                let part_regs: Result<Vec<_>, _> = parts.iter().map(|p| {
                    let r = self.fresh_var();
                    self.gen_expr_to_reg(p, &r, prefix)?;
                    Ok::<_, CodeGenError>(r)
                }).collect();
                let part_regs = part_regs?;
                self.emitf(format!("    %replicate {}, {}, {};",
                    reg, count_reg, part_regs.join(", ")));
            }
            ElabExpr::SystemCall(name, args) => {
                self.gen_system_task(name, args, prefix)?;
                self.emitf(format!("    %vpi_result {};", reg));
            }
            ElabExpr::FuncCall(name, args) => {
                for (i, arg) in args.iter().enumerate() {
                    let r = format!("_farg_{}", i);
                    self.gen_expr_to_reg(arg, &r, prefix)?;
                }
                self.emitf(format!("    %call/func \"{}\", {};", name, reg));
            }
            ElabExpr::HiZ => {
                self.emitf(format!("    %pushi {}, 0, 1; # HiZ/X/Z represented as 0", reg));
            }
        }
        Ok(())
    }
}

// ---- Utility functions ----

/// Format an integer as a bit string for VVP constants
fn format_bits(val: u64, width: u32) -> String {
    if width == 0 { return "0".to_string(); }
    let mut bits = String::new();
    for i in (0..width).rev() {
        bits.push(if (val >> i) & 1 == 1 { '1' } else { '0' });
    }
    bits
}
