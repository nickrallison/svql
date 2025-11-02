// examples/fixtures/basic/logic_tree/verilog/simple_2level.v
// Simple 2-level tree: AND of two ORs
// Structure: y = (a | b) & (c | d)
// Depth: 2, Gates: 3, Leaves: 4

module simple_2level
(
    input a,
    input b,
    input c,
    input d,
    output y
);

wire or1, or2;

assign or1 = a | b;
assign or2 = c | d;
assign y = or1 & or2;

endmodule