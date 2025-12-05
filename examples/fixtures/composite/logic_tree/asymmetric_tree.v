
module asymmetric_tree
(
    input a, b, c, d, e, f,
    output y
);

wire and1, and2, and_left;
assign and1 = a & b;
assign and2 = c & d;
assign and_left = and1 & and2;

wire and_right;
assign and_right = e & f;

assign y = and_left | and_right;

endmodule