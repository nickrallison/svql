// examples/fixtures/basic/logic_tree/verilog/xor_chain.v
// Chain of XOR gates (linear tree)
// Structure: y = ((a ^ b) ^ c) ^ d
// Tests linear/skewed tree structure

module xor_chain
(
    input a, b, c, d,
    output y
);

wire xor1, xor2;

assign xor1 = a ^ b;
assign xor2 = xor1 ^ c;
assign y = xor2 ^ d;

endmodule