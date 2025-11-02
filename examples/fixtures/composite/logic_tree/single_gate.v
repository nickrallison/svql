// examples/fixtures/basic/logic_tree/verilog/single_gate.v
// Single gate (leaf case)
// Structure: y = a & b
// Depth: 1, Gates: 1, Leaves: 2

module single_gate
(
    input a, b,
    output y
);

assign y = a & b;

endmodule