# rust0computer -- 用 rust 重建簡易電腦工業

## 語言與格式

* rust0 -- 簡化後的 rust 語言，副檔名為 .rs
* js0 -- 簡化後的 JavaScript 語言，副檔名為 .js
* ll0 -- 簡化後的 LLVM IR 文字格式，副檔名為 .ll
* qd0 -- 動態語言虛擬機，採用 quadruple 四元組格式，副檔名為 .qd

## 實作工具

* compiler -- 編譯器
    * [x] rust0c -- rust0 之編譯器，類似 rustc
    * [x] ll0vm -- ll0 中間碼虛擬機
    * [x] js0c -- js0 之編譯器
    * [x] qd0c -- qd0 轉為 ll 的編譯器
    * [x] qd0vm -- qd0 的虛擬機
    * [x] qd0lib -- qd0 的指令呼叫與函式庫
    * [ ] ll0c -- 簡化後的 LLVM IR 中間碼組譯器，類似 llc
* ai -- 人工智慧實作
    * [x] nn -- 神經網路套件，類似 rust candle.
    * [x] llm -- 語言模型，類似 GPT
    * [ ] agent -- 代理人，類似 OpenClaw
* os -- 作業系統
    * [ ] kernel -- 用 rust 寫的 RISCV 處理器上之作業系統，類似 xv6
* cpu -- 處理器
    * [ ] RISCV 處理器之 Verilog0 原始碼
* net -- 網路相關
    * [x] telnet -- 重新實作 telnet
    * [ ] browser -- 簡易瀏覽器
* eda -- 電子設計開發工具
    * [ ] verilog0 -- 簡化的 verilog0 語言之模擬器，類似 verilator, icarus verilog.

## 參考

* os
    * https://github.com/o8vm/octox
* ai
    * nn -- https://github.com/huggingface/candle
    * agent -- https://github.com/zeroclaw-labs/zeroclaw

