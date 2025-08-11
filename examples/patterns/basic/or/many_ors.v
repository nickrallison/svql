
module many_ors
(
input a,
input b,
input c,
output y
);

assign y = (((a | b) | c) | d);
endmodule