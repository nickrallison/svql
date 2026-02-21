module full_adder(
    input a,
    input b,
    input cin,
    output sum,
    output cout
);
    wire s1, c1, c2;

    // First Half Adder instance
    half_adder ha1 (
        .a(a),
        .b(b),
        .sum(s1),
        .carry(c1)
    );

    // Second Half Adder instance
    half_adder ha2 (
        .a(s1),
        .b(cin),
        .sum(sum),
        .carry(c2)
    );

    // Carry out logic
    assign cout = c1 | c2;

endmodule

module half_adder(
    input a,
    input b,
    output sum,
    output carry
);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule