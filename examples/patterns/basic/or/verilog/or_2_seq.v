
module or_2_seq
(
input a,
input b,
input c,
output y
);

assign y = ((a | b) | c);
endmodule