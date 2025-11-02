// examples/fixtures/basic/logic_tree/verilog/deep_3level.v
// 3-level nested tree
// Structure: y = ((a & b) | (c & d)) ^ ((e | f) & (g | h))
// Depth: 3, Gates: 9

module deep_3level
(
    input a, b, c, d, e, f, g, h,
    output y
);

// Level 1 (leaves)
wire and1, and2, or1, or2;
assign and1 = a & b;
assign and2 = c & d;
assign or1 = e | f;
assign or2 = g | h;

// Level 2
wire or_left, and_right;
assign or_left = and1 | and2;
assign and_right = or1 & or2;

// Level 3 (root)
assign y = or_left ^ and_right;

endmodule