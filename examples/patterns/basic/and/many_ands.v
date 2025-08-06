
module many_ands
(
input a,
input b,
input c,
output y
);

assign y = (((a & b) & c) & d);
endmodule