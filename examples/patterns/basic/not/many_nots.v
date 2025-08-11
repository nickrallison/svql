
module many_nots
(
input a,
input b,
output y1,
output y2,
);

assign y1 = ~a;
assign y2 = ~b;
endmodule