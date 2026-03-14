## ccc: 完整的版本太大，不好實作，我們能列出一個中間版本，盡可能包含 rust 常用程式的語法，但不會太大的嗎？放在 doc/EBNF2.md 裡

## ccc: 這個語法寫出來的程式，用目前的 rustc 都可以編譯過嗎？我要的是子集合，要能相容的

很好的問題。**現在這份 `EBNF2` 不是保證「寫得出來就一定能被 `rustc` 編過」**，它是「實作導向的近似語法」，所以還**缺少一些 Rust 的語意限制**與**部分語法細節**。結論是：

**不保證 100% 相容 rustc。**  
它更像是「可實作的中間語法」，但不是「嚴格可被 rustc 接受的子集合」。

要讓它成為**真正可相容 rustc 的子集合**，還需要補上或收斂這些點（例子）：

- `*expr`（解參照）在 rustc 需要指標型別，目前語法沒約束型別語意。
- `for` 需要 `IntoIterator`，語意限制沒寫進 EBNF。
- `match` / `if` / `block` 的分號與表達式結尾規則需更精確。
- 結構體/枚舉字面量、`return`、`break`、`continue` 等語法未納入或未細化。
- Rust 的 path / module / visibility / attributes / lifetimes / generics 都沒含。

---

如果你要的是：  
**「子集合，且語法上寫出的程式 `rustc` 一定能編」**  
我可以幫你把 `EBNF2` **縮成真正相容 rustc 的安全子集**，並加上明確語意限制與 edition（例如 Rust 2021 或 2024）。

你希望我**把 `EBNF2.md` 改成「rustc 相容子集合」版本**嗎？  
如果要，我會：

1. 刪掉語意不易保證的語法（如 `match`、`for`、`*expr`）
2. 僅保留「rustc 一定可編」的結構（fn/let/if/while/算術/呼叫）
3. 在文檔中加上清楚的語意約束（型別、return、塊規則等）

告訴我你要哪個方向，我就直接改。

## ccc: 好，刪掉語意不易保證的語法（如 match、for、*expr），僅保留「rustc 一定可編」的結構（fn/let/if/while/算術/呼叫），在文檔中加上清楚的語意約束（型別、return、塊規則等）

已經依你的要求重寫成「**rustc 相容子集合**」版本，放在這裡：

`/Users/cccimac/Desktop/ccc/github/aiopencomputer/compiler/doc/EBNF2.md`

內容已做到：
- **刪除** `match`、`for`、`*expr` 等語意不易保證的語法
- **僅保留** `fn/let/if/while/算術/呼叫`
- **加入明確語意限制**（型別、return、block 規則等）

如果你想把 `while` 也完全移除（讓語意更簡單、保證更強），我也可以再縮一版。

## ccc: 請寫一些符合 EBNF2.md 的 rust 程式範例，放在 compiler/rs/v0.2/ 資料夾中，要能完整測試上述語法，涵蓋每個關鍵字

...

## ccc: 接下來我要寫一版支援 EBNF2.md 的 compiler 和 vm ，請先寫出你的構想方案

這個方向很清楚也很有意思，我先給一份「第二版（v0.2）」的整體構想方案，涵蓋 compiler 與 vm 的設計分工、支援範圍、主要檔案與實作節奏，對齊 `EBNF2.md` 的子集合目標。

**目標**
- Compiler + VM 共同支援 `EBNF2.md`（Rust 相容子集合）
- 產出的 `.ll` 仍由 VM 直接執行（不依賴 LLVM）
- 完整跑過 `compiler/rs/v0.2/` 的範例

---

## 整體架構

### Compiler（Rust → VM IR → `.ll`）
**流程**
1. Lexer：關鍵字、符號、識別字、整數、布林
2. Parser：EBNF2 的語法（`fn/let/if/while/表達式/呼叫/block`）
3. Sema：型別檢查 + 作用域 + 表達式型別一致性
4. IR Lowering：AST → VM IR
5. LLVM IR Emitter：VM IR → `.ll`（只用 VM 支援的指令）

**核心策略**
- `let` 變數 → `alloca` + `store` + `load`
- `if` 表達式 → `icmp` + `br` + `phi`
- `while` → loop blocks + br（但目前語法只允許作為語句，不需要回傳值）
- 所有表達式皆有「值」（`i32` 或 `bool`），`bool` 用 `i1`

---

### VM（`.ll` → 解譯執行）
**流程**
1. Loader：解析 `.ll` 子集（含 `alloca/load/store/phi`）
2. Linker：多檔案連結（符號表）
3. Interpreter：
   - 指令執行
   - stack slots（local allocas）
   - `phi` 前驅選擇
   - `call` + 遞迴

**核心策略**
- `alloca` → 在 Memory slots 建立一個指標名稱
- `store/load` → 從 slots 讀寫
- `phi` → 用上一個 basic block 名稱做分支選擇

---

## 編譯器檔案規劃（v0.2）

- `src/lexer.rs`  
  - 擴充 token：`while`、`||`、`&&`、`!`、`/`、`%`
- `src/parser.rs`  
  - 支援 `while`、logical 表達式、`/` `%`
  - block 規則：`{ stmt* expr }`
- `src/sema.rs`  
  - `while` cond 必須是 `bool`
  - `ExprStmt` 中 `while` 允許（語意定義）
  - 所有 ops 的型別一致
- `src/codegen.rs`  
  - `while` 產生 loop block 結構（cond → body → cond / exit）
  - `&&` `||` 先走簡單 eager 評估（後續可改成短路）
- `src/ir.rs`  
  - 加上 `Div/Rem/And/Or/Not` 或用現有 `ICmp`/`Add` 等組合

---

## VM 檔案規劃（v0.2）

- `src/ir.rs`  
  - 解析 `sdiv/urem` 或 `div/rem`（對應 compiler 輸出）
  - 解析 `and/or/xor`（若 compiler 需要）
- `src/interpreter.rs`  
  - 加上 `div/rem` 指令
  - 布林運算（`and/or/xor/not`）
- `src/memory.rs`  
  - 支援 `i1` slots（目前是 `Value::I1` 已有）

---

## 需要確認的關鍵設計點
1. **`while` 只當語句**還是也允許當表達式？  
   我建議目前只作語句（符合 EBNF2 的語意約束）。
2. **`&&`/`||` 是否先做短路？**  
   可先做 eager（左右都求值），之後再加短路（需要 block 分裂與 phi）。
3. **`/` `%` 的行為**  
   先採整數除法（跟 Rust 一致）。

---

## 開發順序（推薦）
1. Compiler 先補 `while` + `&&/||/!` + `/` `%`
2. VM 補對應指令支援
3. 用 `compiler/rs/v0.2/` 全跑
4. 再處理 `&&/||` 短路、`while` 更完整的 block 結構

---

如果你同意這個方案，我下一步就開始：
1. 補 compiler lexer/parser/sema/codegen
2. 補 vm IR parser + interpreter
3. 用 `v0.2` 範例整批測試

你要我先從 **compiler** 還是 **vm** 開始？