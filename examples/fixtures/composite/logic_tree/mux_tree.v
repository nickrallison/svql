// examples/fixtures/basic/logic_tree/verilog/mux_tree.v
// Tree with MUX gates
// Structure: y = sel1 ? (sel2 ? a : b) : (sel3 ? c : d)
// Tests MUX as tree nodes (3 inputs each)

module mux_tree
(
    input a, b, c, d,
    input sel1, sel2, sel3,
    output y
);

wire mux_left, mux_right;

assign mux_left = sel2 ? a : b;
assign mux_right = sel3 ? c : d;
assign y = sel1 ? mux_left : mux_right;

endmodule