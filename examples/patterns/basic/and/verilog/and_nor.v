// examples/patterns/basic/and/and_nor.v
module and_nor
(
    input  a,
    input  b,
    output y
);

wire nor_out;
wire not_a, not_b;

assign not_a = ~a;
assign not_b = ~b;

assign nor_out = not_a | not_b;

assign y = ~nor_out;

endmodule