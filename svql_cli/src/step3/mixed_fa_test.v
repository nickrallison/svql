module mixed_fa_test(
    input [1:0] a, b, 
    input cin, 
    output [1:0] sum, 
    output cout
);
    wire c_mid;

    // Match 1: Hierarchical Implementation
    // This will be found by the 'Hierarchical' arm of the variant
    full_adder_hier fa_inst1 (
        .a(a[0]),
        .b(b[0]),
        .cin(cin),
        .sum(sum[0]),
        .cout(c_mid)
    );

    // Match 2: Flat Implementation
    // This will be found by the 'Flat' arm of the variant
    full_adder_flat fa_inst2 (
        .a(a[1]),
        .b(b[1]),
        .cin(c_mid),
        .sum(sum[1]),
        .cout(cout)
    );
endmodule

// Helper module for hierarchical match
module full_adder_hier(input a, b, cin, output sum, cout);
    wire s1, c1, c2;
    half_adder ha1(.a(a), .b(b), .sum(s1), .carry(c1));
    half_adder ha2(.a(s1), .b(cin), .sum(sum), .carry(c2));
    assign cout = c1 | c2;
endmodule

// Helper module for hierarchical match
module half_adder(input a, b, output sum, carry);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule

// Module for flat match
module full_adder_flat(input a, b, cin, output sum, cout);
    assign sum = a ^ b ^ cin;
    assign cout = (a & b) | (cin & (a ^ b));
endmodule