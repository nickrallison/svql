// examples/fixtures/basic/logic_tree/verilog/asymmetric_tree.v
// Asymmetric tree (different depths on branches)
// Structure: y = ((a & b) & (c & d)) | (e & f)
// Left branch: depth 2, Right branch: depth 1

module asymmetric_tree
(
    input a, b, c, d, e, f,
    output y
);

// Left branch (deeper)
wire and1, and2, and_left;
assign and1 = a & b;
assign and2 = c & d;
assign and_left = and1 & and2;

// Right branch (shallower)
wire and_right;
assign and_right = e & f;

// Root
assign y = and_left | and_right;

endmodule