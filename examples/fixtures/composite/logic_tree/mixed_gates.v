// examples/fixtures/basic/logic_tree/verilog/mixed_gates.v
// Mixed gate types at same level
// Structure: y = (a & b) | (c ^ d) | (~e)
// Tests OR with different child gate types

module mixed_gates
(
    input a, b, c, d, e,
    output y
);

wire and_out, xor_out, not_out;

assign and_out = a & b;
assign xor_out = c ^ d;
assign not_out = ~e;

assign y = and_out | xor_out | not_out;

endmodule