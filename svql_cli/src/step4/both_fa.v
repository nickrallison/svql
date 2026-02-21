module both_fa(input a, b, cin, output sum, cout);
    wire s1, c1, c2;
    
    // ha1 is a module instance (Structural Variant)
    half_adder ha1(.a(a), .b(b), .sum(s1), .carry(c1));

    // ha2 is a '+' operator (Primitive Variant)
    assign {c2, sum} = s1 + cin;

    assign cout = c1 | c2;
endmodule

module half_adder(input a, b, output sum, carry);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule