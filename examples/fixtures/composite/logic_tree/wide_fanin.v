// examples/fixtures/basic/logic_tree/verilog/wide_fanin.v
// Wide fan-in at single level
// Structure: y = (a & b) | (c & d) | (e & f) | (g & h)
// Tests many children at same depth

module wide_fanin
(
    input a, b, c, d, e, f, g, h,
    output y
);

wire and1, and2, and3, and4;

assign and1 = a & b;
assign and2 = c & d;
assign and3 = e & f;
assign and4 = g & h;

assign y = and1 | and2 | and3 | and4;

endmodule