module mixed_ha(input a, b, c, d, output [1:0] out1, output [1:0] out2);
    // Match 1: Structural HalfAdder (Netlist)
    half_adder ha_inst(
        .a(a),
        .b(b),
        .sum(out1[0]),
        .carry(out1[1])
    );

    // Match 2: Primitive AdcWithCarry ($add)
    // Slicing the result into a 2-bit wire allows AdcWithCarry to match
    assign out2 = c + d;
endmodule

module half_adder(input a, b, output sum, carry);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule