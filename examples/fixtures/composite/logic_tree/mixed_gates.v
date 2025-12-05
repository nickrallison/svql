module mixed_gates
(
    input a, b, c, d, e,
    output y
);

wire and_out, xor_out, not_out;

assign and_out = a & b;
assign xor_out = c ^ d;
assign not_out = ~e;

assign y = and_out | xor_out | not_out;

endmodule