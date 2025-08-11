
module many_ands
(
input a,
input p,
input q,
input r,
input b,
output y
);

assign y = (((a & b) & p) & q) & r;

endmodule