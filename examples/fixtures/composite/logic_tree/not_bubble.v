// examples/fixtures/basic/logic_tree/verilog/not_bubble.v
// Inverters at multiple levels
// Structure: y = ~((~a & ~b) | (c & d))
// Tests NOT gates scattered through tree

module not_bubble
(
    input a, b, c, d,
    output y
);

wire not_a, not_b, and1, and2, or_out;

assign not_a = ~a;
assign not_b = ~b;
assign and1 = not_a & not_b;
assign and2 = c & d;
assign or_out = and1 | and2;
assign y = ~or_out;

endmodule