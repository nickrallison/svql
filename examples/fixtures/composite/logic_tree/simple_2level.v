
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