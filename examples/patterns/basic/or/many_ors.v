
module many_ors
(
input a,
input b,
input c,
input d,
input e,
output y
);

assign y = (((a | b) | c) | d) | e;
endmodule