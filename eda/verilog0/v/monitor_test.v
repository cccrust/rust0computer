// monitor_test.v
module monitor_test;
    reg clk;
    reg [3:0] counter; // 第一次挑戰多位元寬度 (Bit-width)

    // 簡易的 Clock 產生器 (不用 always 區塊，用 initial + 遞迴/迴圈)
    // 第一版為了避開 always，可以用這種寫法
    initial begin
        clk = 1'b0;
        forever #5 clk = ~clk; 
    end

    initial begin
        counter = 4'b0000;
        // 註冊 monitor：任何參數改變都會印出
        $monitor("Time=%0t | clk=%b | counter=%d", $time, clk, counter);
        
        #10 counter = 4'b0001;
        #10 counter = 4'b0010;
        #15 $finish;
    end
endmodule