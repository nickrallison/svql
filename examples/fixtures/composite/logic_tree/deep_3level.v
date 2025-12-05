
module deep_3level
(
    input a, b, c, d, e, f, g, h,
    output y
);

wire and1, and2, or1, or2;
assign and1 = a & b;
assign and2 = c & d;
assign or1 = e | f;
assign or2 = g | h;

wire or_left, and_right;
assign or_left = and1 | and2;
assign and_right = or1 & or2;

assign y = or_left ^ and_right;

endmodule