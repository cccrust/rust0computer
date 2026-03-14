// tb_delay.v
module tb_delay;
    reg sig;

    initial begin
        sig = 1'b0;
        $display("Time %0t: sig = %b", $time, sig);
        
        #10; // 推進 10 個時間單位
        sig = 1'b1;
        $display("Time %0t: sig = %b", $time, sig);
        
        #20; // 再推進 20 個時間單位
        sig = 1'b0;
        $display("Time %0t: sig = %b", $time, sig);
        
        #5;
        $finish; // 結束模擬
    end
endmodule
