// top.v (假設與半加器寫在一起或預設 Include)
module top;
    reg test_a;
    reg test_b;
    wire test_sum;
    wire test_carry;

    // 實例化 half_adder，測試按名字映射 (.port(signal))
    half_adder u0 (
        .a(test_a),
        .b(test_b),
        .sum(test_sum),
        .carry(test_carry)
    );

    initial begin
        // 窮舉測試
        test_a = 0; test_b = 0;
        #10;
        $display("a=%b, b=%b -> sum=%b, carry=%b", test_a, test_b, test_sum, test_carry);
        
        test_a = 0; test_b = 1;
        #10;
        $display("a=%b, b=%b -> sum=%b, carry=%b", test_a, test_b, test_sum, test_carry);

        test_a = 1; test_b = 1;
        #10;
        $display("a=%b, b=%b -> sum=%b, carry=%b", test_a, test_b, test_sum, test_carry);

        $finish;
    end
endmodule
