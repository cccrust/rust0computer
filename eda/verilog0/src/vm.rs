use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum Inst {
    Pushi(String, i64),       // %pushi T1, 0, 8
    LoadVec4(String, String), // %load/vec4 T0, vcounter/rst
    AssignNB(String, String), // %assign/nb count, 0, T1
    Add(String, String, String), // %add T2, T3, T4
    WaitPosedge(String),      // %wait/posedge clk
    Delay(u64),               // #delay
    Jmp(String),              // %jmp label
    JmpFalse(String, String), // %jmp/false T0, label
    VpiCall(String),          // %vpi_call "$display"
    End,
    NoOp,
}

#[derive(Clone)]
struct Thread {
    pc: usize,
    regs: HashMap<String, i64>,
    is_blocked: bool,
    waiting_on: Option<String>, // 等待訊號名
    last_val: i64,             // 用於邊緣偵測的舊值
}

#[derive(Eq, PartialEq)]
struct ScheduledEvent {
    time: u64,
    thread_idx: usize,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering { other.time.cmp(&self.time).reverse() }
}
impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub struct Vm {
    memory: HashMap<String, i64>,
    code: Vec<Inst>,
    labels: HashMap<String, usize>,
    threads: Vec<Thread>,
    event_queue: BinaryHeap<ScheduledEvent>,
    nba_queue: VecDeque<(String, i64)>, // Non-blocking assignment queue
    current_time: u64,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            memory: HashMap::new(),
            code: Vec::new(),
            labels: HashMap::new(),
            threads: Vec::new(),
            event_queue: BinaryHeap::new(),
            nba_queue: VecDeque::new(),
            current_time: 0,
        }
    }

    pub fn load_from_string(&mut self, vvp_code: &str) -> Result<(), String> {
        for line in vvp_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(':') || line.starts_with('#') { continue; }

            let mut content = line;
            if let Some(idx) = line.find(':') {
                let label = line[..idx].trim().to_string();
                self.labels.insert(label, self.code.len());
                content = line[idx+1..].trim();
            }

            if content.is_empty() { continue; }

            let inst = if content.starts_with(".thread") {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if parts.len() > 1 {
                    let label = parts[1].replace('@', "");
                    // 這裡簡單化：假設在 load 完後手動啟動 thread
                }
                Inst::NoOp
            } else if content.contains("%pushi") {
                let parts: Vec<&str> = content.split(',').collect();
                let reg = parts[0].replace("%pushi", "").trim().to_string();
                let val = parts[1].trim().parse().unwrap_or(0);
                Inst::Pushi(reg, val)
            } else if content.contains("%load/vec4") {
                let parts: Vec<&str> = content.split(',').collect();
                let reg = parts[0].replace("%load/vec4", "").trim().to_string();
                let var = parts[1].trim().to_string();
                Inst::LoadVec4(reg, var)
            } else if content.contains("%wait/posedge") {
                let var = content.replace("%wait/posedge", "").trim().to_string();
                Inst::WaitPosedge(var)
            } else if content.contains("%assign/nb") {
                let parts: Vec<&str> = content.split(',').collect();
                let var = parts[0].replace("%assign/nb", "").trim().to_string();
                let reg = parts[2].trim().to_string();
                Inst::AssignNB(var, reg)
            } else if content.contains("%jmp/false") {
                let parts: Vec<&str> = content.split(',').collect();
                let reg = parts[0].replace("%jmp/false", "").trim().to_string();
                let label = parts[1].trim().replace('@', "");
                Inst::JmpFalse(reg, label)
            } else if content.contains("%vpi_call") {
                Inst::VpiCall(content.to_string())
            } else {
                Inst::NoOp
            };
            self.code.push(inst);
        }
        // 手動掃描 .thread 並加入初始化 (這部分在實際 parser 應更嚴謹)
        self.threads.push(Thread { pc: 0, regs: HashMap::new(), is_blocked: false, waiting_on: None, last_val: 0 });
        self.event_queue.push(ScheduledEvent { time: 0, thread_idx: 0 });
        Ok(())
    }

    pub fn run(&mut self) {
        println!("--- VM Simulation Started ---");
        
        while let Some(event) = self.event_queue.pop() {
            self.current_time = event.time;
            let tid = event.thread_idx;
            self.threads[tid].is_blocked = false;

            // 執行當前 thread 直到遇到阻塞指令
            loop {
                if self.threads[tid].pc >= self.code.len() { break; }
                let inst = self.code[self.threads[tid].pc].clone();
                self.threads[tid].pc += 1;

                match inst {
                    Inst::Pushi(reg, val) => { self.threads[tid].regs.insert(reg, val); }
                    Inst::LoadVec4(reg, var) => {
                        let val = *self.memory.get(&var).unwrap_or(&0);
                        self.threads[tid].regs.insert(reg, val);
                    }
                    Inst::AssignNB(var, reg) => {
                        let val = *self.threads[tid].regs.get(&reg).unwrap_or(&0);
                        self.nba_queue.push_back((var, val));
                    }
                    Inst::WaitPosedge(var) => {
                        let cur_val = *self.memory.get(&var).unwrap_or(&0);
                        self.threads[tid].waiting_on = Some(var);
                        self.threads[tid].last_val = cur_val;
                        self.threads[tid].is_blocked = true;
                        break;
                    }
                    Inst::JmpFalse(reg, label) => {
                        if *self.threads[tid].regs.get(&reg).unwrap_or(&0) == 0 {
                            self.threads[tid].pc = *self.labels.get(&label).unwrap_or(&0);
                        }
                    }
                    Inst::VpiCall(s) => {
                        if s.contains("$display") || s.contains("$monitor") {
                            println!("Time {}: count = {}", self.current_time, self.memory.get("count").unwrap_or(&0));
                        }
                    }
                    _ => {}
                }
            }

            // 處理非阻塞賦值 (Update Phase)
            let mut changed = false;
            while let Some((var, val)) = self.nba_queue.pop_front() {
                if self.memory.get(&var) != Some(&val) {
                    self.memory.insert(var, val);
                    changed = true;
                }
            }

            // 如果有訊號改變，檢查是否有 thread 正在等待該訊號
            if changed {
                for i in 0..self.threads.len() {
                    if let Some(ref var) = self.threads[i].waiting_on {
                        let new_val = *self.memory.get(var).unwrap_or(&0);
                        if self.threads[i].last_val == 0 && new_val == 1 { // Posedge!
                            self.threads[i].waiting_on = None;
                            self.event_queue.push(ScheduledEvent { time: self.current_time, thread_idx: i });
                        }
                    }
                }
            }
        }
        println!("--- VM Simulation Ended ---");
    }
}