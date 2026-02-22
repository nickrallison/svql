module mixed_ha_test(input a, b, c, d, output out1, carry1, out2, carry2);
    // Match 1: Structural HalfAdder (Netlist)
    half_adder ha_inst(
        .a(a),
        .b(b),
        .sum(out1),
        .carry(out1)
    );

    wire [1:0] out2_b;

    // Match 2: Primitive AdcWithCarry ($add)
    // Slicing the result into a 2-bit wire allows AdcWithCarry to match
    assign out2_b = c + d;
    assign out2 = out2_b[0];
    assign carry2 = out2_b[1];
endmodule

module half_adder(input a, b, output sum, carry);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule