// half_adder.v
module half_adder(
    input wire a,
    input wire b,
    output wire sum,
    output wire carry
);

    // 測試 XOR 和 AND 運算子
    assign sum = a ^ b;
    assign carry = a & b;

endmodule