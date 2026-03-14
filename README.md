# rust0 -- 用 rust 重建簡易電腦工業

* compiler -- 編譯器
    * [x] rust0c -- 簡化後的 RUST 語言 RUST0 之編譯器
    * [x] ir0vm -- 簡化後的 LLVM IR 中間碼虛擬機
    * [x] js0c -- 簡化後的 JavaScript 語言 js0 之編譯器
    * [ ] ir0as -- 簡化後的 LLVM IR 中間碼組譯器 
* ai -- 人工智慧實作
    * [x] nn -- 神經網路套件，類似 rust candle.
    * [x] llm -- 語言模型，類似 GPT
    * [ ] agent -- 代理人，類似 OpenClaw
* os
    * [ ] kernel -- 用 rust 寫的 RISCV 處理器上之作業系統
* cpu
    * [ ] RISCV 處理器之 Verilog0 原始碼
* net -- 網路相關
    * [ ] telnet -- 重新實作 telnet
    * [ ] browser -- 簡易瀏覽器
* eda -- 電子設計開發工具
    * [ ] verilog0 -- 簡化的 verilog0 語言之模擬器

## 參考

* os
    * https://github.com/o8vm/octox
* nn
    * https://github.com/huggingface/candle
